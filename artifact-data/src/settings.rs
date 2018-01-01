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
//! #SPC-data-settings
//! This contains the logic for loading the settings of an artifact project.

use toml;
use serde::{self, Deserialize, Deserializer, Serialize, Serializer};
use std::sync::mpsc::{channel, Sender};

use dev_prelude::*;
use path_abs::{discover_paths, PathAbs};
use lint;

pub const SETTINGS_PATH: &str = ".art/settings.toml";

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub struct SettingsRaw {
    // pub artifact_paths: OrderSet<PathBuf>,
    // pub exclude_artifact_paths: OrderSet<PathBuf>,
    pub code_paths: Vec<String>,
    pub exclude_code_paths: Vec<String>,
    // pub file_type: FileType,
}

#[derive(Debug, Eq, PartialEq)]
/// All paths that have to be loaded in the project.
pub struct ProjectPaths {
    pub code: OrderSet<PathAbs>,
    // pub artifacts: OrderSet<PathAbs>,
}

/// Load the paths to all files in the project from the root path.
///
/// The settings file is required to be in `project_path/.art/settings.toml`
pub fn load_project_paths(project_path: &PathAbs) -> Result<(ProjectPaths, Vec<lint::Lint>)> {
    let (lints, recv_lints) = channel();
    let mut raw: SettingsRaw = {
        let settings_path = PathAbs::new(project_path.join(SETTINGS_PATH))?;
        toml::from_str(&settings_path.read()?)?
    };

    let mut code_paths: OrderSet<PathAbs> = OrderSet::new();
    let mut visited_code_paths = load_raw_paths(&lints, project_path, &raw.exclude_code_paths);

    for p in raw.code_paths.drain(0..) {
        let base = project_path.join(p);
        let discovered = discover_paths(&base, |_| true, &visited_code_paths);
        let mut paths = match discovered {
            Ok(p) => p,
            Err(err) => {
                lint::io_error(&lints, &base, &err.to_string());
                continue;
            }
        };
        visited_code_paths.extend(paths.files.iter().cloned());
        visited_code_paths.extend(paths.dirs.drain(0..));
        code_paths.extend(paths.files.drain(0..));
    }

    let paths = ProjectPaths { code: code_paths };
    drop(lints);
    let lints: Vec<_> = recv_lints.into_iter().collect();
    Ok((paths, lints))
}

/// Load a list of string paths using the project_path as the base path (i.e. from a settings file)
fn load_raw_paths(
    lints: &Sender<lint::Lint>,
    project_path: &PathAbs,
    raw_paths: &[String],
) -> OrderSet<PathAbs> {
    raw_paths
        .iter()
        .filter_map(|p| {
            // backwards compatibility: just ignore front `{repo}/`
            let p = p.trim_left_matches("{repo}/");
            let path = project_path.join(p);
            match PathAbs::new(&path) {
                Ok(p) => Some(p),
                Err(err) => {
                    lint::io_error(lints, &path, &err.to_string());
                    None
                }
            }
        })
        .collect()
}
