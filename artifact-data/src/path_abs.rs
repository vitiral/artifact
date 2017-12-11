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
    pub fn new(path: &OsStr) -> io::Result<PathAbs> {
        ::cache::PATH_CACHE.lock().unwrap().get(path)
    }

    #[cfg(not(feature = "cache"))]
    pub fn new(path: &OsStr) -> io::Result<PathAbs> {
        return PathAbs(Arc::new(Path::new(path).canonicalize()?));
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
    fn get(&mut self, raw: &OsStr) -> io::Result<PathAbs> {
        if let Some(p) = self.paths.get(raw) {
            return Ok(p.clone());
        }

        let path = PathAbs(Arc::new(Path::new(raw).canonicalize()?));
        self.paths.insert(raw.to_os_string(), path.clone());
        Ok(path)
    }
}
