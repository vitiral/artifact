/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2017  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */
//! The major exported type and function for loading artifacts.

use time;
use std::borrow::Borrow;
use std::sync::mpsc::{channel, Sender};
use rayon;

use dev_prelude::*;
use artifact;
use implemented;
use lint;
use name;
use raw;
use settings;
use project;

#[derive(Debug, PartialEq)]
pub struct Project {
    pub paths: settings::ProjectPaths,
    pub code_impls: OrderMap<name::Name, implemented::ImplCode>,
    pub artifacts: OrderMap<name::Name, artifact::Artifact>,
}

impl Project {
    /// Recursively sort all the items in the project.
    pub fn sort(&mut self) {
        sort_orderset(&mut self.paths.code);
        sort_orderset(&mut self.paths.artifact);

        sort_ordermap(&mut self.code_impls);
        for (_, code) in self.code_impls.iter_mut() {
            sort_ordermap(&mut code.secondary);
        }
        sort_ordermap(&mut self.artifacts);
        for (_, art) in self.artifacts.iter_mut() {
            art.sort();
        }
    }

    /// #SPC-data-lint
    /// FIXME: Need to do LINTS
    /// - partof that doesn't exist
    /// - parent that should exist but doesn't
    /// - check for type validity in "partof"
    /// - code_impls can NOT conflict with done
    /// - code_impls with no corresponding raw artifact
    /// - code_impls with no corresponding raw artifact subname
    pub fn lint(&self) -> lint::Categorized {
        let mut lints = lint::Categorized::default();

        lints.sort();
        lints
    }
}

/// Load the project from the given path.
pub fn load_project<P: AsRef<Path>>(project_path: P) -> (lint::Categorized, Option<Project>) {
    let start_load = time::get_time();
    let mut lints = lint::Categorized::default();

    let paths = {
        let start = time::get_time();
        let (mut load_lints, paths) = settings::load_project_paths(project_path);
        lints.categorize(load_lints.drain(..));
        if !lints.error.is_empty() {
            lints.sort();
            return (lints, None);
        }
        eprintln!("loaded paths in {:.3} seconds", time::get_time() - start);
        paths.expect("No lints but also no settings file!")
    };

    let (code_impls, (defined, loaded)) = {
        let (send, recv) = channel();
        let send = Mutex::new(send);
        let (locs, arts) = rayon::join(
            || {
                let start = time::get_time();
                let send = { send.lock().map(|s| s.clone()).unwrap() };
                let locs = implemented::load_locations(&send, &paths.code);
                let out = implemented::join_locations(&send, locs);
                eprintln!("loaded code in {:.3} seconds", time::get_time() - start);
                out
            },
            || {
                let start = time::get_time();
                let send = { send.lock().map(|s| s.clone()).unwrap() };
                let raw = raw::load_artifacts_raw(&send, &paths.artifact);
                let (defined, raw) = raw::join_artifacts_raw(&send, raw);
                let loaded = artifact::finalize_load_artifact(raw);
                eprintln!(
                    "loaded artifacts in {:.3} seconds",
                    time::get_time() - start
                );
                (defined, loaded)
            },
        );
        let start = time::get_time();
        drop(send);
        lints.categorize(recv.into_iter());
        eprintln!(
            "categorized load-lints in {:.3} seconds",
            time::get_time() - start
        );

        if !lints.error.is_empty() {
            lints.sort();
            return (lints, None);
        }
        (locs, arts)
    };

    let start = time::get_time();
    let artifacts = artifact::determine_artifacts(loaded, &code_impls, &defined);

    let mut project = Project {
        paths: paths,
        code_impls: code_impls,
        artifacts: artifacts,
    };
    println!(
        "determined project in {:.3} seconds",
        time::get_time() - start
    );
    let start = time::get_time();
    lints.sort();
    project.sort();
    eprintln!("sorted project in {:.3} seconds", time::get_time() - start);
    eprintln!(
        "project load took {:.3} seconds",
        time::get_time() - start_load
    );
    (lints, Some(project))
}
