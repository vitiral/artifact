/*  rst: the requirements tracking tool made for developers
    Copyright (C) 2016  Garrett Berg <@vitiral, vitiral@gmail.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the Lesser GNU General Public License as published 
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the Lesser GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/
use std::path::{Path, PathBuf};
use std::collections::{HashMap, HashSet};

use time;


// General
pub mod types;

// for loading
pub mod utils;
#[macro_use] pub mod load;  // macro use so the macro can be tested
pub mod vars;
pub mod link;
pub mod locs;

// serialization
#[cfg(feature = "serde_codegen")]
include!(concat!(env!("OUT_DIR"), "/serde_types.rs"));

#[cfg(feature = "serde_derive")] 
include!("serde_types.in.rs");
mod serialize;

#[cfg(test)]
mod tests;

// export for other modules to use
pub use core::utils::find_repo;
pub use core::types::{
    LoadResult, LoadError,
    Project,
    Artifact, Artifacts,
    ArtType, Loc,
    ArtName, ArtNameRc, ArtNames,
    Settings, LoadFromStr,
    PARENT_PATH};
pub use core::load::load_toml;

#[cfg(test)]
use super::init_logger;
#[cfg(test)]
pub fn init_logger_test() {
    match init_logger(false, 3, false) {
        Ok(_) => {},
        Err(_) => {},
    }
}

/// Load all items from the toml file at path
pub fn load_path(path: &Path) 
        -> LoadResult<Project>{
    let start = time::get_time();
    info!("loading path: {}", path.to_string_lossy().as_ref());
    let mut project = try!(load::load_raw(path));

    info!("resolving and filling variables");
    try!(vars::fill_text_fields(
        &mut project.artifacts, &mut project.variables, 
        &mut project.repo_map));

    info!("finding and attaching locations");
    let locs = try!(locs::find_locs(&mut project.settings));
    project.dne_locs = locs::attach_locs(&mut project.artifacts, locs);

    // do all links
    try!(link::do_links(&mut project.artifacts));
    let total = time::get_time() - start;
    info!("Done loading: {} artifacts loaded successfullly in {:.3} seconds",
          project.artifacts.len(), total.num_milliseconds() as f64 * 1e-3);
    Ok(project)
}

