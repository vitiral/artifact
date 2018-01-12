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
use std::sync::mpsc::{channel, Sender};
use rayon;
use regex::Regex;

use dev_prelude::*;
use artifact;
use implemented;
use lint;
use name::{Name, SubName};
use raw;
use settings;
use path_abs::PathAbs;

#[derive(Debug, PartialEq)]
pub struct Project {
    pub paths: settings::ProjectPaths,
    pub code_impls: OrderMap<Name, implemented::ImplCode>,
    pub artifacts: OrderMap<Name, artifact::Artifact>,
}

lazy_static!{
    /// Name reference that can exist in source code
    static ref TEXT_REF_RE: Regex = Regex::new(
        &format!(r#"(?xi)
        \[\[(               # start main section
        (?:REQ|SPC|TST)     # all types are supported
        -(?:[{0}]+-)*       # any number of first element
        (?:[{0}]+)          # required end element
        )                   # end main section
        (\.[{0}]+)?         # (optional) sub section
        \]\]                # close text reference
        "#, NAME_VALID_CHARS!())).unwrap();
}

impl Project {
    /// Recursively sort all the items in the project.
    pub fn sort(&mut self) {
        self.paths.code.sort();
        self.paths.artifact.sort();

        self.code_impls.sort_keys();
        for (_, code) in self.code_impls.iter_mut() {
            code.secondary.sort_keys();
        }
        self.artifacts.sort_keys();
        for (_, art) in self.artifacts.iter_mut() {
            art.sort();
        }
    }

    /// #SPC-read-lint
    ///
    /// TODO WARN:
    /// - references in text that do not exist
    /// - (optional?) poorly formed references in text
    pub fn lint(&self) -> lint::Categorized {
        let (send, recv) = channel();

        self.lint_errors(&send);
        self.lint_other(&send);

        drop(send);
        let mut lints = lint::Categorized::default();
        lints.categorize(recv.into_iter());
        lints.sort();
        lints
    }

    /// Lint against only "fatal" errors.
    pub fn lint_errors(&self, send: &Sender<lint::Lint>) {
        lint_partof_dne(send, self);
        lint_partof_types(send, self);
        lint_artifact_text(send, self);
        lint_artifact_done_subnames(send, self);
    }

    /// Lint against non-fatal errors.
    pub fn lint_other(&self, send: &Sender<lint::Lint>) {
        lint_artifact_text_refs(send, self);
        lint_code_impls(send, self);
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

/// #REQ-family.lint_partof_exists
/// Lint against partofs that do not exist but should (ERROR)
pub(crate) fn lint_partof_dne(lints: &Sender<lint::Lint>, project: &Project) {
    for (name, art) in project.artifacts.iter() {
        for pof in art.partof.iter() {
            if !project.artifacts.contains_key(pof) {
                lints
                    .send(lint::Lint {
                        level: lint::Level::Error,
                        path: Some(art.file.to_path_buf()),
                        line: None,
                        category: lint::Category::Artifact,
                        msg: format!(
                            "{} defines partof={} which does not exist",
                            name.as_str(),
                            pof.as_str()
                        ),
                    })
                    .expect("send lint");
            }
        }
    }
}

/// #REQ-family.lint_types
/// Lint against partof's that have invalid types.
pub(crate) fn lint_partof_types(lints: &Sender<lint::Lint>, project: &Project) {
    use name::Type::{REQ, SPC, TST};
    for (name, art) in project.artifacts.iter() {
        for pof in art.partof.iter() {
            let invalid = match (name.ty, pof.ty) {
                // SPC can not have part REQ
                (REQ, SPC)
                // TST can not have part REQ
                | (REQ, TST)
                // TST can not have part SPC
                | (SPC, TST) => true,
                _ => false,
            };

            if invalid {
                lints
                    .send(lint::Lint {
                        level: lint::Level::Error,
                        path: Some(art.file.to_path_buf()),
                        line: None,
                        category: lint::Category::Artifact,
                        msg: format!(
                            "{} cannot have `partof` {}: invalid types.",
                            name.as_str(),
                            pof.as_str(),
                        ),
                    })
                    .expect("send lint");
            }
        }
    }
}

/// #SPC-read-artifact.lint_done
/// Lint that done is not defined on an artifact which has subnames.
pub(crate) fn lint_artifact_done_subnames(lints: &Sender<lint::Lint>, project: &Project) {
    for (name, art) in project.artifacts.iter() {
        if art.impl_.is_done() && !art.subnames.is_empty() {
            lints
                .send(lint::Lint {
                    level: lint::Level::Error,
                    path: Some(art.file.to_path_buf()),
                    line: None,
                    category: lint::Category::Artifact,
                    msg: format!(
                        "{}: subnames are defined when the `done` field is set.",
                        name.as_str()
                    ),
                })
                .expect("send lint");
        }
    }
}

/// Lint against code_impls
pub(crate) fn lint_code_impls(lints: &Sender<lint::Lint>, project: &Project) {
    use implemented::{CodeLoc, Impl};
    let send_lint = |name: &Name, sub: Option<&SubName>, loc: &CodeLoc, msg: &str| {
        lints
            .send(lint::Lint {
                level: lint::Level::Warn,
                path: Some(loc.file.to_path_buf()),
                line: Some(loc.line),
                category: lint::Category::ImplCode,
                msg: format!("Invalid code impl #{}. {}", name.full(sub), msg),
            })
            .expect("send lint");
    };
    for (name, code_impl) in project.code_impls.iter() {
        if let Some(art) = project.artifacts.get(name) {
            match art.impl_ {
                Impl::Done(_) => {
                    // #SPC-read-impl.lint_done
                    // impls exist for artifact defined as done
                    if let Some(ref loc) = code_impl.primary {
                        send_lint(name, None, loc, "Artifact's done field is set");
                    }
                    for (sub, loc) in code_impl.secondary.iter() {
                        send_lint(name, Some(sub), loc, "Artifact's done field is set");
                    }
                }
                Impl::Code(_) => {
                    for (sub, loc) in code_impl.secondary.iter() {
                        if !art.subnames.contains(sub) {
                            // #SPC-read-impl.lint_exists
                            // subname ref does not exist
                            send_lint(
                                name,
                                Some(sub),
                                loc,
                                &format!(
                                    "Subname [[{}]] does not exist in artifact's text",
                                    sub.as_str()
                                ),
                            );
                        }
                    }
                }
                Impl::NotImpl => {}
            }
        } else {
            // #SPC-read-impl.lint_subname_exists
            // artifact does not exist!
            if let Some(ref loc) = code_impl.primary {
                send_lint(
                    name,
                    None,
                    loc,
                    &format!("Artifact {} does not exist", name.as_str()),
                );
            }
            for (sub, loc) in code_impl.secondary.iter() {
                send_lint(
                    name,
                    Some(sub),
                    loc,
                    &format!("Artifact {} does not exist", name.as_str()),
                );
            }
        }
    }
}

/// #SPC-read-artifact.lint_text
/// Lint against artifact text being structured incorrectly.
pub(crate) fn lint_artifact_text(lints: &Sender<lint::Lint>, project: &Project) {
    let send_lint = |name: &Name, file: &PathAbs, msg: &str| {
        lints
            .send(lint::Lint {
                level: lint::Level::Error,
                path: Some(file.to_path_buf()),
                line: None,
                category: lint::Category::Artifact,
                msg: format!("{} text is invalid: {}", name.as_str(), msg),
            })
            .expect("send lint");
    };
    for (name, art) in project.artifacts.iter() {
        for line in art.text.lines() {
            if raw::NAME_LINE_RE.is_match(line) {
                send_lint(
                    name,
                    &art.file,
                    "Cannot have a line of the form \"# ART-name\" as that specifies a new \
                     artifact in the markdown format.",
                )
            } else if raw::ATTRS_END_RE.is_match(line) {
                send_lint(
                    name,
                    &art.file,
                    "Cannot have a line of the form \"###+\" as that specifies \
                     the end of the metadata in the markdown format.",
                )
            }
        }
    }
}

/// #SPC-read-artifact.lint_text_refs
/// Lint warnings against invalid references in the artifact text.
pub(crate) fn lint_artifact_text_refs(lints: &Sender<lint::Lint>, project: &Project) {
    let send_lint = |name: &Name, ref_name: &Name, ref_sub: Option<&SubName>, file: &PathAbs| {
        lints
            .send(lint::Lint {
                level: lint::Level::Warn,
                path: Some(file.to_path_buf()),
                line: None,
                category: lint::Category::Artifact,
                msg: format!(
                    "{} has soft reference [[{}]] which does not exist.",
                    name.as_str(),
                    ref_name.full(ref_sub)
                ),
            })
            .expect("send lint");
    };
    for (name, art) in project.artifacts.iter() {
        for captures in TEXT_REF_RE.captures_iter(&art.text) {
            // unwrap: group 1 always exists in regex
            let name_mat = captures.get(1).unwrap();
            // unwrap: pre-validated by regex
            let ref_name = Name::from_str(name_mat.as_str()).unwrap();
            // subname is optional
            let ref_sub = match captures.get(2) {
                Some(sub_mat) => Some(SubName::new_unchecked(sub_mat.as_str())),
                None => None,
            };
            match (project.artifacts.get(&ref_name), &ref_sub) {
                (None, _) => {
                    // specified an artifact that does not exist
                    send_lint(name, &ref_name, ref_sub.as_ref(), &art.file);
                }
                (Some(ref_art), &Some(ref sub)) => {
                    if !ref_art.subnames.contains(sub) {
                        // specified a sub that does not exist
                        send_lint(name, &ref_name, Some(sub), &art.file);
                    }
                }
                _ => {}
            }
        }
    }
}
