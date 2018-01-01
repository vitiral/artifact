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
//! #SPC-data-cache
//!
//! This defines the caches, currently there are two:
//! - NameCache: cache of name tokens.
//! - PathCache: cache of absolute paths. The logic lives here
use dev_prelude::*;
use name::{Name, Type};
use path_abs::PathAbs;

lazy_static!{
    /// global cache of names
    pub(crate) static ref NAME_CACHE: Mutex<NameCache> = Mutex::new(NameCache {
        names: OrderMap::new(),
    });

    pub(crate) static ref PATH_CACHE: Mutex<PathCache> = Mutex::new(PathCache {
        paths: OrderMap::new(),
    });
}

/// Global cache of names. Note: the methods live in `name.rs`.
///
/// #SPC-data-cache.name
pub(crate) struct NameCache {
    pub(crate) names: OrderMap<String, Name>,
}

/// Global cache of absolute paths. Note: the methods live in `path_abs.rs`.
///
/// #SPC-data-cache.path
pub(crate) struct PathCache {
    /// References made to paths to avoid extra OS calls
    pub(crate) paths: OrderMap<OsString, PathAbs>,
}

/// Clear the internal caches.
///
/// Mosty used for tests to prevent memory from balooning.
pub fn clear_cache() {
    {
        let mut cache = NAME_CACHE.lock().unwrap();
        cache.names.clear();
    }
    {
        let mut cache = PATH_CACHE.lock().unwrap();
        cache.paths.clear();
    }
}
