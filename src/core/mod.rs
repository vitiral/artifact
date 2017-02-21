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
use time;
use dev_prefix::*;

// General
pub mod prefix;
pub mod types;

// for loading
pub mod utils;
#[macro_use]
pub mod load; // macro_use so the macro can be tested
pub mod save;
pub mod link;
pub mod locs;
pub mod name;
pub mod security;

mod serialize;

#[cfg(test)]
pub mod tests;

// export for other modules to use
pub use core::load::load_toml;
pub use core::utils::find_repo;
pub use core::types::{Project, Artifact, Artifacts, ArtType, Loc, ArtName, ArtNameRc, ArtNames,
                      Settings, LoadFromStr, PARENT_PATH, LocData, ArtifactData};

#[cfg(test)]
use super::init_logger;
#[cfg(test)]
pub fn init_logger_test() {
    match init_logger(false, 3, false) {
        Ok(_) => {}
        Err(_) => {}
    }
}

/// This method is for processing a raw project-text
/// file into a full blown project. This is necessary
/// mostly for the API (when it is editing the project)
/// and for unit tests
pub fn process_project_text(settings: Settings,
                            project_text: &types::ProjectText)
                            -> Result<Project> {
    let mut project = Project::default();
    project.settings = settings;
    project.extend_text(project_text)?;
    load::process_project(&mut project)?;
    project.origin = project_text.origin.clone();
    Ok(project)
}

/// Load all items from the toml file at path
/// #SPC-load
pub fn load_path(path: &Path) -> Result<Project> {
    let start = time::get_time();
    info!("loading path: {}", path.to_string_lossy().as_ref());
    let project = load::load_project(path)?;

    let total = time::get_time() - start;
    info!("Done loading: {} artifacts loaded successfullly in {:.3} seconds",
          project.artifacts.len(),
          total.num_milliseconds() as f64 * 1e-3);
    Ok(project)
}
