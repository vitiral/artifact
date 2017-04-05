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
//! This module defines the loading and saving of artifacts from user data

use time;
use dev_prefix::*;
use types::*;
use security;

mod types;
mod utils;
mod save;
mod link;
mod locs;
mod name;
mod project;
mod serialize;
mod artifact;
mod settings;

#[cfg(test)]
mod tests;

// export for other modules to use
pub use user::save::{ProjectText, PathDiff};

// Exposed for testing only
#[cfg(test)]
pub use user::artifact::{load_toml, load_text as load_project_text};
#[cfg(test)]
pub use user::link::do_links;
#[cfg(test)]
pub use user::settings::from_table as settings_from_table;


/// Process a raw project text file.
///
/// This can be called on a project which has not yet
/// had it's links completed.
pub fn process_project(project: &mut Project) -> Result<()> {
    let locs = locs::find_locs(&project.settings)?;
    project.dne_locs = locs::attach_locs(&mut project.artifacts, locs)?;
    link::do_links(&mut project.artifacts)?;
    Ok(())
}

/// This method is for processing a raw project-text
/// file into a full blown project.
///
/// This is mostly used for validation that nothing has
/// changed in converting the project.
///
/// This method should be considered unstable.
pub fn process_project_text(settings: Settings, project_text: &ProjectText) -> Result<Project> {
    let mut project = Project::default();
    project.settings = settings;
    artifact::extend_text(&mut project, project_text)?;
    process_project(&mut project)?;
    project.origin = project_text.origin.clone();
    Ok(project)
}

/// Load all items from the toml file in the repo
/// #SPC-load
pub fn load_repo(repo: &Path) -> Result<Project> {
    let start = time::get_time();
    info!("loading path: {}", repo.display());

    let settings = settings::load_settings(repo)?;
    security::validate_settings(repo, &settings)?;

    let mut project_text = ProjectText::default();
    let mut loaded_paths: HashSet<PathBuf> = HashSet::new();
    loaded_paths.insert(repo.join(REPO_DIR.as_path()));
    loaded_paths.extend(settings.exclude_artifact_paths.iter().cloned());

    for path in &settings.artifact_paths {
        if loaded_paths.contains(path) {
            continue;
        }
        loaded_paths.insert(path.to_path_buf());
        artifact::load_text(&mut project_text, path.as_path(), &mut loaded_paths)?;
    }

    let mut project = Project::default();
    project.settings = settings.clone();
    artifact::extend_text(&mut project, &project_text)?;

    process_project(&mut project)?;
    project.origin = repo.to_path_buf();

    let total = time::get_time() - start;
    info!("Done loading: {} artifacts loaded successfullly in {:.3} seconds",
          project.artifacts.len(),
          total.num_milliseconds() as f64 * 1e-3);
    Ok(project)
}
