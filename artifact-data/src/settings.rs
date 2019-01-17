/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018 Rett Berg <@vitiral, vitiral@gmail.com>
 *
 * The source code is Licensed under either of
 *
 * * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
 *   http://www.apache.org/licenses/LICENSE-2.0)
 * * MIT license ([LICENSE-MIT](LICENSE-MIT) or
 *   http://opensource.org/licenses/MIT)
 *
 * at your option.
 *
 * Unless you explicitly state otherwise, any contribution intentionally submitted
 * for inclusion in the work by you, as defined in the Apache-2.0 license, shall
 * be dual licensed as above, without any additional terms or conditions.
 * */
//! This contains the logic for loading the settings of an artifact project.
use ergo::toml;

use dev_prelude::*;
use raw;
use std::io;

pub const ART_DIR: &str = ".art";
pub const SETTINGS_FILE: &str = "settings.toml";

#[derive(Debug, Eq, PartialEq, Clone, Serialize, Deserialize)]
pub(crate) struct SettingsRaw {
    pub artifact_paths: Vec<String>,
    pub exclude_artifact_paths: Vec<String>,
    pub code_paths: Vec<String>,
    pub exclude_code_paths: Vec<String>,
    pub code_url: Option<String>,

    #[serde(default)]
    pub export: SettingsExport,
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
    ) -> ::std::result::Result<(PathDir, PathFile, SettingsRaw), String> {
        let project_path = PathDir::new(project_path.as_ref()).map_err(|e| {
            format!(
                "folder does not exist at {}, got {}",
                project_path.as_ref().display(),
                e
            )
        })?;
        let expected = project_path.join(ART_DIR).join(SETTINGS_FILE);
        let settings_path = PathFile::new(&expected).map_err(|e| e.to_string())?;

        let raw: SettingsRaw = {
            let text = settings_path.read_string().map_err(|e| e.to_string())?;
            toml::from_str(&text).map_err(|e| e.to_string())?
        };

        debug!("SettingsRaw={:#?}", raw);
        Ok((project_path, settings_path, raw))
    }
}

pub(crate) fn walk_artifact_paths(
    send_paths: &Sender<PathFile>,
    send_err: &Sender<lint::Lint>,
    paths: &IndexSet<PathAbs>,
    exclude_paths: &IndexSet<PathAbs>,
) {
    let f = |path: &PathType| -> bool {
        let abs: &PathAbs = path.as_ref();
        !(exclude_paths.contains(abs)
            || (path.is_file() && raw::ArtFileType::from_path(path).is_none()))
    };
    walk_paths(send_paths, send_err, paths, f)
}

pub(crate) fn walk_paths<F>(
    send_paths: &Sender<PathFile>,
    send_err: &Sender<lint::Lint>,
    paths: &IndexSet<PathAbs>,
    filter: F,
) where
    F: Fn(&PathType) -> bool,
{
    for path in paths.iter() {
        let res = walk_path(send_paths, path.clone(), &filter);
        if let Err(err) = res {
            ch!(send_err <- lint::Lint::load_error(path.to_string(), &err.to_string()));
        }
    }
}

/// Walk the path, using the filter and sending any found files
/// to the sender.
///
/// `filter` filters out every item that return `false`
fn walk_path<F>(send_paths: &Sender<PathFile>, path: PathAbs, filter: &F) -> io::Result<()>
where
    F: Fn(&PathType) -> bool,
{
    let dir = match PathType::from_abs(path)? {
        PathType::File(f) => {
            ch!(send_paths <- f);
            return Ok(());
        }
        PathType::Dir(dir) => dir,
    };
    let mut it = dir.walk().into_iter();
    loop {
        let entry = match it.next() {
            None => break,
            Some(e) => e?,
        };

        let ty = PathType::from_entry(entry)?;

        if !filter(&ty) {
            if ty.is_dir() {
                it.skip_current_dir();
            }
            continue;
        }

        if let PathType::File(file) = ty {
            ch!(send_paths <- file);
        }
    }
    Ok(())
}

/// Load the paths to all files in the project from the root path.
///
/// The settings file is required to be in `project_path/.art/settings.toml`
pub(crate) fn load_settings<P: AsRef<Path>>(
    project_path: P,
) -> (Vec<lint::Lint>, Option<Settings>) {
    let (project_path, settings_path, raw) = match SettingsRaw::load(project_path.as_ref()) {
        Ok(s) => s,
        Err(err) => {
            let lints = vec![lint::Lint::load_error(
                project_path.as_ref().to_string_lossy(),
                &err.to_string(),
            )];
            return (lints, None);
        }
    };

    let (send_lints, recv_lints) = ::std::sync::mpsc::channel();
    let paths = Settings {
        base: project_path.clone(),
        settings_path: settings_path,
        code_paths: resolve_raw_paths(&send_lints, &project_path, &raw.code_paths),
        exclude_code_paths: resolve_raw_paths(&send_lints, &project_path, &raw.exclude_code_paths),
        artifact_paths: resolve_raw_paths(&send_lints, &project_path, &raw.artifact_paths),
        exclude_artifact_paths: resolve_raw_paths(
            &send_lints,
            &project_path,
            &raw.exclude_artifact_paths,
        ),
        code_url: raw.code_url,

        export: raw.export,
    };
    drop(send_lints);
    let lints = recv_lints.into_iter().collect();
    (lints, Some(paths))
}

/// Load a list of string paths using the `project_path` as the base path (i.e. from a settings file)
fn resolve_raw_paths(
    lints: &::std::sync::mpsc::Sender<lint::Lint>,
    project_path: &PathAbs,
    raw_paths: &[String],
) -> IndexSet<PathAbs> {
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
                    lint::io_error(lints, path.to_string(), &err.to_string());
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
    visited: &IndexSet<PathAbs>,
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
