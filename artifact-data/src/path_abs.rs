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

use dev_prelude::*;

use std::io;
use std::fmt;
use std::ffi::OsStr;

#[derive(Clone)]
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
}

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


#[cfg(test)]
mod test {
    use std::fs;
    use tempdir;

    use super::*;
    use test::dev_prelude::*;

    #[test]
    fn sanity_path_abs() {
        // make the directory inside of target
        let tmp = tempdir::TempDir::new_in("target", "path-abs-").unwrap();

        // paths that we will create
        let dir1 = tmp.path().join("dir1");
        let d1_f1 = dir1.join("f1");
        let d1_f2 = dir1.join("f2");
        let dir2 = tmp.path().join("dir2");
        let d2_f1 = dir2.join("f1");

        // paths that we do not create
        let dne1 = tmp.path().join("dne1");
        let dne2 = dir1.join("dne2");
        let dne3 = dir2.join("dne3");

        for p in &[&dir1, &dir2] {
            fs::create_dir(p).unwrap()
        }

        for f in &[&d1_f1, &d1_f2, &d2_f1] {
            touch(f).unwrap();
        }

        // find the just created paths (3 times for testing cache)
        let mut paths: Vec<PathAbs> = Vec::new();
        for _ in 0..3 {
            for p in &[&dir1, &dir2, &d1_f1, &d1_f2, &d2_f1] {
                paths.push(PathAbs::new(p).unwrap())
            }
        }

        // paths that do not exist are errors
        for p in &[&dne1, &dne2, &dne3] {
            PathAbs::new(p).unwrap_err();
        }
    }
}
