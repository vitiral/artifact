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
//! #SPC-read-settings
//! This contains the logic for loading the settings of an artifact project.

use toml;
use std::sync::mpsc::{channel, Sender};
use path_abs::{PathAbs, PathFile};
use walkdir::WalkDir;

use dev_prelude::*;
use raw::FileType;
use lint;

pub const SETTINGS_PATH: &str = ".art/settings.toml";

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct SettingsRaw {
    pub artifact_paths: Vec<String>,
    pub exclude_artifact_paths: Vec<String>,
    pub code_paths: Vec<String>,
    pub exclude_code_paths: Vec<String>,
    // pub file_type: FileType,
}

pub(crate) struct FoundPaths {
    pub files: Vec<PathFile>,
    pub dirs: Vec<PathAbs>,
}

impl FoundPaths {
    pub(crate) fn new() -> FoundPaths {
        FoundPaths {
            files: Vec::new(),
            dirs: Vec::new(),
        }
    }
}

impl SettingsRaw {
    fn load<P: AsRef<Path>>(
        project_path: P,
    ) -> ::std::result::Result<(PathAbs, SettingsRaw), String> {
        let project_path = PathAbs::new(project_path.as_ref()).map_err(|e| {
            format!(
                "folder does not exist at {}, got {}",
                project_path.as_ref().display(),
                e
            )
        })?;
        let raw: SettingsRaw = {
            let expected = project_path.join(SETTINGS_PATH);
            let settings_path = PathFile::new(&expected).map_err(|e| e.to_string())?;
            let text = settings_path.read_string().map_err(|e| e.to_string())?;
            toml::from_str(&text).map_err(|e| e.to_string())?
        };
        Ok((project_path, raw))
    }
}

#[derive(Debug, Eq, PartialEq)]
/// All paths that have to be loaded in the project.
pub struct ProjectPaths {
    pub base: PathAbs,
    pub code: OrderSet<PathFile>,
    pub artifact: OrderSet<PathFile>,
}

/// Load the paths to all files in the project from the root path.
///
/// The settings file is required to be in `project_path/.art/settings.toml`
pub(crate) fn load_project_paths<P: AsRef<Path>>(
    project_path: P,
) -> (Vec<lint::Lint>, Option<ProjectPaths>) {
    let (project_path, raw) = match SettingsRaw::load(project_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            let lints = vec![
                lint::Lint::load_error(project_path.as_ref(), &err.to_string()),
            ];
            return (lints, None);
        }
    };

    let (send_lints, recv_lints) = channel();
    let paths = ProjectPaths {
        base: project_path.clone(),
        code: discover_settings_paths(
            &send_lints,
            &project_path,
            &raw.code_paths,
            &raw.exclude_code_paths,
            // TODO: add ability to exclude file patterns
            &|_| true,
        ),
        artifact: discover_settings_paths(
            &send_lints,
            &project_path,
            &raw.artifact_paths,
            &raw.exclude_artifact_paths,
            &|p| FileType::from_path(p.as_path()).is_some(),
        ),
    };
    drop(send_lints);
    let lints = recv_lints.into_iter().collect();
    (lints, Some(paths))
}

/// Discover a list of paths from the settings file.
fn discover_settings_paths<F>(
    lints: &Sender<lint::Lint>,
    project_path: &PathAbs,
    raw_paths: &[String],
    raw_exclude: &[String],
    filter: &F,
) -> OrderSet<PathFile>
where
    F: Fn(&PathAbs) -> bool,
{
    let mut discovered: OrderSet<PathFile> = OrderSet::new();
    let mut visited = resolve_raw_paths(lints, project_path, raw_exclude);

    for base in resolve_raw_paths(lints, project_path, raw_paths) {
        let paths = discover_paths(base.as_path(), filter, &visited);
        let mut paths = match paths {
            Ok(p) => p,
            Err(err) => {
                lint::io_error(lints, base.as_path(), &err.to_string());
                continue;
            }
        };
        visited.extend(paths.files.iter().map(|p| p.clone().into()));
        visited.extend(paths.dirs.drain(..).map(|p| p.into()));
        discovered.extend(paths.files.drain(..));
    }
    discovered
}

/// Load a list of string paths using the `project_path` as the base path (i.e. from a settings file)
fn resolve_raw_paths(
    lints: &Sender<lint::Lint>,
    project_path: &PathAbs,
    raw_paths: &[String],
) -> OrderSet<PathAbs> {
    raw_paths
        .iter()
        .filter_map(|p| {
            // backwards compatibility: just ignore front `{repo}/`
            let p = p.trim_left_matches("{repo}");
            // Also just allow `/something`... Path.join will just IGNORE joining
            // something with the other being "/something"
            let p = p.trim_left_matches('/');
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

/// Walk the path returning the found files and directories.
///
/// `filter` is a closure to filter file (not dir) names. Return `false` to exclude
/// the file from `files`.
///
/// It is expected that the caller will add the visited directories
/// to the `visited` parameter for the next call to avoid duplicated
/// effort.
pub(crate) fn discover_paths<F, P>(
    path: P,
    filter: &F,
    visited: &OrderSet<PathAbs>,
) -> ::std::io::Result<FoundPaths>
where
    P: AsRef<Path>,
    F: Fn(&PathAbs) -> bool,
{
    let mut found = FoundPaths::new();
    let mut it = WalkDir::new(path).into_iter();
    loop {
        let entry = match it.next() {
            None => break,
            Some(e) => e?,
        };

        let abs = PathAbs::new(entry.path())?;
        let filetype = entry.file_type();

        if visited.contains(&abs) {
            if filetype.is_dir() {
                it.skip_current_dir();
            }
            continue;
        }

        if filetype.is_dir() {
            found.dirs.push(abs);
        } else {
            debug_assert!(filetype.is_file());
            if !filter(&abs) {
                continue;
            }
            found.files.push(abs.into_file()?);
        }
    }
    Ok(found)
}

