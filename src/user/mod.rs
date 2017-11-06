/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
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
mod artifact;
mod subname;
mod settings;
mod markdown;

#[cfg(test)]
mod tests;

// export for other modules to use
pub use user::save::{PathDiff, ProjectText};

// Exposed for testing only
#[cfg(test)]
pub use user::artifact::{load_file_path, load_text};
#[cfg(test)]
pub use user::link::do_links;
#[cfg(test)]
pub use user::settings::{from_raw as settings_from_raw, from_text as settings_from_text};
#[cfg(test)]
pub use user::types::RawSettings;


/// Process a raw project text file.
///
/// This can be called on a project which has not yet
/// had it's links completed.
///
/// #SPC-project-process
pub fn process_project(project: &mut Project) -> Result<()> {
    let (locs, sublocs) = locs::find_locs(&project.settings)?;
    let dne = locs::attach_locs(&mut project.artifacts, locs, sublocs)?;
    project.dne_locs = dne.0;
    project.dne_sublocs = dne.1;
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
///
/// #SPC-project-load
pub fn load_repo(repo: &Path) -> Result<Project> {
    let start = time::get_time();
    info!("Loading path: {}", repo.display());

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
        artifact::load_file_path(&mut project_text, path.as_path(), &mut loaded_paths)?;
    }

    let mut project = Project::default();
    project.settings = settings.clone();
    artifact::extend_text(&mut project, &project_text)?;

    process_project(&mut project)?;
    project.origin = repo.to_path_buf();

    let total = time::get_time() - start;
    info!(
        "Done loading: {} artifacts loaded successfullly in {:.3} seconds",
        project.artifacts.len(),
        total.num_milliseconds() as f64 * 1e-3
    );
    Ok(project)
}
