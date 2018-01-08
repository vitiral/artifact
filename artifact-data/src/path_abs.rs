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

/// Functions for handling paths in an "absolute" manner.

use std::io;
use std::fmt;
use walkdir::WalkDir;

use dev_prelude::*;

// ------------------------------
// -- EXPORTED TYPES / METHODS
#[derive(Clone, Eq, Hash, PartialEq, PartialOrd, Ord)]
pub struct PathAbs(Arc<PathBuf>);

impl PathAbs {
    #[cfg(feature = "cache")]
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<PathAbs> {
        ::cache::PATH_CACHE.lock().unwrap().get(path)
    }

    #[cfg(not(feature = "cache"))]
    pub fn new<P: AsRef<Path>>(path: P) -> io::Result<PathAbs> {
        Ok(PathAbs(Arc::new(path.as_ref().canonicalize()?)))
    }

    /// Read the contents of a file
    pub fn read(&self) -> io::Result<String> {
        let mut f = File::open(self.as_path())?;
        let mut out = String::new();
        f.read_to_string(&mut out)?;
        Ok(out)
    }

    /// Create a path from a "project path", i.e. from a settings file.
    ///
    /// Note: for backwards compatibility, project paths can have "{repo}/" in front of them.
    pub fn new_project(project_path: &PathAbs, raw_path: &str) -> io::Result<PathAbs> {
        // backwards compatibility: just ignore front `{repo}/`
        let raw_path = raw_path.trim_left_matches("{repo}/");
        PathAbs::new(&project_path.join(raw_path))
    }

    /// For constructing fake paths during tests
    pub fn fake<P: AsRef<Path>>(fake_path: P) -> PathAbs {
        PathAbs(Arc::new(fake_path.as_ref().to_path_buf()))
    }
}

pub(crate) struct FoundPaths {
    pub files: Vec<PathAbs>,
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
            found.files.push(abs);
        }
    }
    Ok(found)
}

// ------------------------------
// -- IMPLEMENTATION DETAILS

impl fmt::Debug for PathAbs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.0)
    }
}

impl Deref for PathAbs {
    type Target = PathBuf;

    fn deref(&self) -> &PathBuf {
        self.0.as_ref()
    }
}

#[cfg(feature = "cache")]
impl ::cache::PathCache {
    /// Get the path from the cache, inserting it if it doesn't exist
    ///
    /// This is the only way that paths are ever referenced.
    fn get<P: AsRef<Path>>(&mut self, raw: P) -> io::Result<PathAbs> {
        let os_str = raw.as_ref().as_os_str();
        if let Some(p) = self.paths.get(os_str) {
            return Ok(p.clone());
        }

        let path = PathAbs(Arc::new(raw.as_ref().canonicalize()?));
        self.paths.insert(os_str.to_os_string(), path.clone());
        Ok(path)
    }
}

// ------------------------------
// -- TESTS

#[cfg(test)]
mod test {
    use std::fs;
    use tempdir;

    use super::*;

    impl FoundPaths {
        /// Used for testing comparisons
        pub fn sort(&mut self) {
            self.files.sort();
            self.dirs.sort();
        }
    }

    #[test]
    fn sanity_path_abs() {
        // make the directory inside of target
        let tmp = tempdir::TempDir::new_in("target", "path-abs-").unwrap();

        // paths that we will create
        let f1 = tmp.path().join("f1");
        let dir1 = tmp.path().join("dir1");
        let d1_f1 = dir1.join("f1");
        let d1_f2 = dir1.join("f2");
        let dir2 = tmp.path().join("dir2");
        let d2_f1 = dir2.join("f1");

        let dirs_raw = &[&dir1, &dir2];
        let files_raw = &[&f1, &d1_f1, &d1_f2, &d2_f1];

        for p in dirs_raw.iter() {
            fs::create_dir(p).unwrap()
        }

        for f in files_raw.iter() {
            touch(f).unwrap();
        }

        let mut dirs: Vec<_> = dirs_raw.iter().map(|p| PathAbs::new(p).unwrap()).collect();
        let mut files: Vec<_> = files_raw.iter().map(|p| PathAbs::new(p).unwrap()).collect();
        dirs.sort();
        files.sort();

        let f1_abs = PathAbs::new(&f1).unwrap();
        let d1_f2_abs = PathAbs::new(&d1_f2).unwrap();
        let tmp_abs = PathAbs::new(tmp.path()).unwrap();

        let mut expected_dirs = dirs.clone();
        expected_dirs.push(tmp_abs.clone());
        expected_dirs.sort();

        // make sure loading works as expected
        {
            let mut found = discover_paths(tmp.path(), &|_| true, &OrderSet::new()).unwrap();
            found.sort();
            assert_eq!(found.files, files);
            assert_eq!(found.dirs, expected_dirs);
        }

        // visiting no directories because they are already visited
        {
            let visited = OrderSet::from_iter(dirs.iter().map(|p| p.clone()));
            let found = discover_paths(tmp.path(), &|_| true, &visited).unwrap();
            assert_eq!(found.files, &[f1_abs.clone()]);
            assert_eq!(found.dirs, &[tmp_abs.clone()]);
        }

        // filtering out files named f1
        {
            let filter_names = hashset!{OsString::from("f1")};
            let filter = |p: &PathAbs| {
                // if it is contained return False (i.e. do not let it exist)
                !filter_names.contains(p.file_name().unwrap())
            };
            let mut found = discover_paths(tmp.path(), &filter, &OrderSet::new()).unwrap();
            found.sort();
            assert_eq!(found.files, &[d1_f2_abs.clone()]);
            assert_eq!(found.dirs, expected_dirs);
        }
    }
}
