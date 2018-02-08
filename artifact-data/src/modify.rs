/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018  Garrett Berg <@vitiral, vitiral@gmail.com>
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
use artifact;
use intermediate::{ArtifactIm, HashIm};
use lint;
use name::Name;
use project::{read_project, Project};
use raw;
use settings;

static ART_BK_EXT: &str = ".artbk";

#[derive(Debug, Serialize, Deserialize)]
/// #SPC-structs.artifact_op
/// Used for specifying operations to perform.
pub enum ArtifactOp {
    Create {
        artifact: ArtifactIm,
    },
    Update {
        artifact: ArtifactIm,
        orig_id: HashIm,
    },
    Delete {
        name: Name,
        orig_id: HashIm,
    },
}

impl ArtifactOp {
    pub(crate) fn clean(&mut self) {
        match *self {
            ArtifactOp::Create { ref mut artifact }
            | ArtifactOp::Update {
                ref mut artifact, ..
            } => artifact.clean(),
            _ => {}
        }
    }
}

#[derive(Debug)]
pub struct ModifyError {
    lints: lint::Categorized,
    kind: ModifyErrorKind,
}

#[derive(Debug)]
pub enum ModifyErrorKind {
    /// Project was corrupted by the user
    InvalidFromLoad,

    /// Some of the hash ids did not match
    HashMismatch,

    /// The project would have been corrupted by the modifications
    InvalidFromModify,

    /// Failure while creating, recovery required.
    CreateBackups,

    /// Failure while saving the project, recovery required.
    SaveProject,
}

/// Perform a list of modifications to the project
pub fn modify_project<P: AsRef<Path>>(
    project_path: P,
    operations: Vec<ArtifactOp>,
) -> ::std::result::Result<(lint::Categorized, Project), ModifyError> {
    let (mut lints, original_project) = read_project(project_path);

    macro_rules! check_lints {
        ($kind:ident) => {
            if !lints.error.is_empty() {
                lints.sort();
                return Err(ModifyError {
                    lints: lints,
                    kind: ModifyErrorKind::$kind,
                });
            }
        };
    }

    let original_project = match original_project {
        Some(p) => p,
        None => {
            check_lints!(InvalidFromLoad);
            unreachable!()
        }
    };
    let mut artifacts = original_project.artifacts;

    let mut artifact_ims: OrderMap<HashIm, ArtifactIm> = artifacts
        .drain(..)
        .map(|(_, art)| {
            let im = ArtifactIm::from(art);
            (im.hash_im(), im)
        })
        .collect();

    perform_operations(operations, &mut lints, &mut artifact_ims);
    check_lints!(HashMismatch);

    let artifacts_im = artifact_ims.drain(..).map(|(_, a)| a).collect();
    let (send_errs, recv_errs) = ch::unbounded();
    let (defined, raw) = raw::join_artifacts_raw(&send_errs, artifacts_im);
    let loaded = artifact::finalize_load_artifact(raw);
    let artifacts = artifact::determine_artifacts(loaded, &original_project.code_impls, &defined);

    let mut project = Project {
        paths: original_project.paths,
        code_impls: original_project.code_impls,
        artifacts: artifacts,
    };
    project.lint_errors(&send_errs);

    drop(send_errs);
    lints.categorize(recv_errs.iter());
    check_lints!(InvalidFromModify);

    create_backups(&mut lints, project.paths.clone());
    check_lints!(CreateBackups);

    save_project(&mut lints, &project);
    check_lints!(SaveProject);

    remove_backups(&mut lints, project.paths.clone());
    project.sort();

    Ok((lints, project))
}

fn perform_operations(
    mut operations: Vec<ArtifactOp>,
    lints: &mut lint::Categorized,
    artifact_ims: &mut OrderMap<HashIm, ArtifactIm>,
) {
    for mut op in operations.drain(..) {
        op.clean();
        match op {
            ArtifactOp::Create { artifact } => {
                let hash = artifact.hash_im();
                if let Some(exists) = artifact_ims.insert(hash, artifact) {
                    lints.error.push(lint::Lint::create_exists(format!(
                        "Attempting to create an artifact which already exists: {:?}",
                        exists
                    )));
                }
            }
            ArtifactOp::Update { artifact, orig_id } => {
                let hash = artifact.hash_im();
                if artifact_ims.remove(&orig_id).is_none() {
                    lints.error.push(lint::Lint::update_dne(format!(
                        "Attempt to update '{}' failed, hash-id does not exist",
                        artifact.name.as_str(),
                    )));
                } else {
                    artifact_ims.insert(hash, artifact);
                }
            }
            ArtifactOp::Delete { name, orig_id } => {
                if artifact_ims.remove(&orig_id).is_none() {
                    lints.error.push(lint::Lint::delete_dne(format!(
                        "Attempt to delete '{}' failed, hash-id does not exist",
                        name.as_str(),
                    )));
                }
            }
        }
    }
}

/// #SPC-modify.backup
fn create_backups(lints: &mut lint::Categorized, paths: Arc<settings::ProjectPaths>) {
    let recv_lint = {
        let (send_lint, recv_lint) = ch::bounded(128);
        let (send_path, recv_path) = ch::bounded(128);
        take!(=send_lint as errs);
        spawn(move || {
            settings::walk_artifact_paths(
                &send_path,
                &errs,
                &paths.artifact_paths,
                &paths.exclude_artifact_paths,
            )
        });

        for _ in 0..num_cpus::get() {
            take!(=recv_path, =send_lint);
            spawn(move || {
                for path in recv_path {
                    let bk = path.with_extension(ART_BK_EXT);
                    if let Err(err) = path.clone().rename(bk) {
                        let l = lint::Lint {
                            level: lint::Level::Error,
                            path: Some(path.into()),
                            line: None,
                            category: lint::Category::CreateBackups,
                            msg: err.to_string(),
                        };
                        ch!(send_lint <- l);
                    }
                }
            });
        }

        recv_lint
    };

    lints.categorize(recv_lint.iter());
}

fn remove_backups(lints: &mut lint::Categorized, paths: Arc<settings::ProjectPaths>) {
    let recv_lint = {
        let (send_lint, recv_lint) = ch::bounded(128);
        let (send_path, recv_path) = ch::bounded(128);
        take!(=send_lint as errs);
        spawn(move || {
            settings::walk_paths(&send_path, &errs, &paths.artifact_paths, |path| {
                let abs: &PathAbs = path.as_ref();
                if path.is_dir() || path.extension() == Some(OsStr::new(ART_BK_EXT)) {
                    // only include "backup" files or any directories not in exclude
                    !paths.exclude_artifact_paths.contains(abs)
                } else {
                    false
                }
            })
        });

        for _ in 0..num_cpus::get() {
            take!(=recv_path, =send_lint);
            spawn(move || {
                for path in recv_path {
                    if let Err(err) = path.clone().remove() {
                        let l = lint::Lint {
                            level: lint::Level::Warn,
                            path: Some(path.into()),
                            line: None,
                            category: lint::Category::RemoveBackups,
                            msg: err.to_string(),
                        };
                        ch!(send_lint <- l);
                    }
                }
            });
        }

        recv_lint
    };

    lints.categorize(recv_lint.iter());
}

/// Save the project to disk, recording any lints along the way
fn save_project(lints: &mut lint::Categorized, project: &Project) {
    // split up the artifacts into their relevant files
    let mut files: OrderMap<PathFile, OrderMap<Name, raw::ArtifactRaw>> = OrderMap::new();
    for art in project.artifacts.values() {
        let art = ArtifactIm::from(art.clone());
        let (file, name, raw) = art.into_raw();
        let entry = files.entry(file).or_insert_with(OrderMap::new);
        entry.insert(name, raw);
    }

    let recv_lint = {
        let (send_lint, recv_lint) = ch::bounded(128);
        let (send_arts, recv_arts) = ch::bounded(128);

        for _ in 0..num_cpus::get() {
            take!(=send_lint, =recv_arts);
            spawn(move || {
                for (file, arts) in recv_arts {
                    let file: PathFile = file;
                    // FIXME: use settings to specfiy the type of file to save
                    let text = raw::to_markdown(&arts);
                    if let Err(err) = file.write_str(&text) {
                        let l = lint::Lint {
                            level: lint::Level::Error,
                            path: Some(file.into()),
                            line: None,
                            category: lint::Category::SaveProject,
                            msg: err.to_string(),
                        };
                        ch!(send_lint <- l);
                    }
                }
            });
        }

        for arts in files.drain(..) {
            ch!(send_arts <- arts);
        }

        recv_lint
    };

    lints.categorize(recv_lint.iter());
}
