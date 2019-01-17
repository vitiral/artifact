//! #SPC-modify

use std::error;
use std::fmt;

use artifact;
use dev_prelude::*;
use intermediate::ArtifactImExt;
use project::{read_project, ProjectExt};
use raw;
use settings;

static ART_BK_EXT: &str = "artbk";

/// Perform a list of modifications to the project
pub fn modify_project<P: AsRef<Path>>(
    project_path: P,
    mut operations: Vec<ArtifactOp>,
) -> ::std::result::Result<(lint::Categorized, Project), ModifyError> {
    macro_rules! check_lints {
        ($lints:ident, $kind:ident) => {
            if !$lints.error.is_empty() {
                $lints.sort();
                return Err(ModifyError {
                    lints: $lints,
                    kind: ModifyErrorKind::$kind,
                });
            }
        };
    }

    let (mut lints, original_project) = match read_project(project_path) {
        Ok(ok) => ok,
        Err(mut lints) => {
            check_lints!(lints, InvalidFromLoad);
            unreachable!()
        }
    };
    check_lints!(lints, InvalidFromLoad);

    // TODO: move this before even reading the project
    check_overlap(&mut lints, &mut operations);
    check_lints!(lints, InvalidPaths);

    check_paths(&mut lints, &original_project, &operations);
    check_lints!(lints, InvalidPaths);

    let mut artifacts = original_project.artifacts;

    let mut artifact_ims: IndexMap<HashIm, ArtifactIm> = artifacts
        .drain(..)
        .map(|(_, art)| {
            let im = ArtifactIm::from(art);
            (im.hash_im(), im)
        })
        .collect();

    perform_operations(operations, &mut lints, &mut artifact_ims);
    check_lints!(lints, HashMismatch);

    let artifacts_im = artifact_ims.drain(..).map(|(_, a)| a).collect();
    let (send_errs, recv_errs) = ch::unbounded();
    let (defined, raw) = raw::join_artifacts_raw(&send_errs, artifacts_im);
    let loaded = artifact::finalize_load_artifact(raw);
    let artifacts = artifact::determine_artifacts(loaded, &original_project.code_impls, &defined);

    let mut project = Project {
        settings: original_project.settings,
        code_impls: original_project.code_impls,
        artifacts: artifacts,
    };
    project.lint_errors(&send_errs);

    drop(send_errs);
    lints.categorize(recv_errs.iter());
    check_lints!(lints, InvalidFromModify);

    create_backups(&mut lints, project.settings.clone());
    check_lints!(lints, CreateBackups);

    save_project(&mut lints, &project);
    check_lints!(lints, SaveProject);

    remove_backups(&mut lints, project.settings.clone());
    project.sort();

    Ok((lints, project))
}

/// Make sure that
///
/// - All file extensions are valid.
/// - None of the requested modifications are outside of the include paths.
fn check_paths(lints: &mut lint::Categorized, project: &Project, operations: &[ArtifactOp]) {
    // TODO: make parallel
    for op in operations.iter() {
        let path = match *op {
            ArtifactOp::Create { ref artifact } => artifact.file.clone(),
            ArtifactOp::Update { ref artifact, .. } => artifact.file.clone(),
            ArtifactOp::Delete { .. } => continue,
        };

        macro_rules! not_valid {
            ($msg:expr) => {{
                let l = lint::Lint {
                    level: lint::Level::Error,
                    path: Some(path.to_string()),
                    line: None,
                    category: lint::Category::ModifyPathInvalid,
                    msg: $msg.to_string(),
                };
                lints.error.push(l);
                continue;
            }};
        }

        if raw::ArtFileType::from_path(&path).is_none() {
            not_valid!("Not one of the valid extensions [.toml, .md, .json]")
        }

        // First make sure that the path _can_ be valid
        let artifact_paths = &project.settings.artifact_paths;
        let valid_paths: Vec<_> = artifact_paths
            .iter()
            .filter_map(|valid| {
                if path.starts_with(valid) {
                    Some(valid)
                } else {
                    None
                }
            })
            .collect();

        if valid_paths.is_empty() {
            not_valid!("Not inside artifact_paths");
        }

        // It COULD be valid, but it could also be excluded
        for exclude in project.settings.exclude_artifact_paths.iter() {
            if path.starts_with(exclude) {
                // okay, the path is inside an exclude path... is there an
                // INCLUDE that is inside this EXCLUDE?
                if valid_paths.iter().any(|valid| valid.starts_with(exclude)) {
                    not_valid!("Is inside exclude_artifact_paths");
                }
            }
        }
    }
}

fn check_overlap(lints: &mut lint::Categorized, operations: &mut Vec<ArtifactOp>) {
    let mut ids = IndexSet::new();

    for mut op in operations {
        op.clean();

        let pieces = op.id_pieces();

        macro_rules! overlap { [$name:expr] => {{
            lints.error.push(lint::Lint::id_overlap(
                format!(
                    "Attempting to operate twice on {}",
                    $name
                )
            ));
        }}}

        if pieces.new_id == pieces.orig_id {
            // They are already identical, NOOP
            continue;
        }

        if let Some(id) = pieces.new_id {
            if !ids.insert(id) {
                overlap!(pieces.name);
            }
        }

        if let Some(id) = pieces.orig_id {
            if !ids.insert(id) {
                overlap!(pieces.name);
            }
        }
    }
}

/// #SPC-modify-update
fn perform_operations(
    mut operations: Vec<ArtifactOp>,
    lints: &mut lint::Categorized,
    artifact_ims: &mut IndexMap<HashIm, ArtifactIm>,
) {
    for op in operations.drain(..) {
        match op {
            ArtifactOp::Create { artifact } => {
                let id = artifact.hash_im();
                if let Some(exists) = artifact_ims.insert(id, artifact) {
                    lints.error.push(lint::Lint::create_exists(format!(
                        "Attempting to create an artifact which already exists: {:?}",
                        exists
                    )));
                }
            }
            ArtifactOp::Update { artifact, orig_id } => {
                let id = artifact.hash_im();
                if artifact_ims.remove(&orig_id).is_none() {
                    lints.error.push(lint::Lint::update_dne(format!(
                        "Attempt to update '{}' failed, hash-id does not exist",
                        artifact.name,
                    )));
                    continue;
                } else {
                    artifact_ims.insert(id, artifact);
                }
            }
            ArtifactOp::Delete { name, orig_id } => {
                match artifact_ims.remove(&orig_id) {
                    None => {
                        lints.error.push(lint::Lint::delete_dne(format!(
                            "Attempt to delete '{}' failed, hash-id does not exist",
                            name,
                        )));
                        continue;
                    }
                    Some(art) => art.name,
                };
            }
        };
    }
}

/// #SPC-modify.backup
fn create_backups(lints: &mut lint::Categorized, paths: Settings) {
    // TODO: figure out how to just use a reference
    let paths = Arc::new(paths);
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
                            path: Some(path.to_string()),
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

fn remove_backups(lints: &mut lint::Categorized, paths: Settings) {
    // TODO: figure out how to just use a reference
    let paths = Arc::new(paths);
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
                            path: Some(path.to_string()),
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
    let mut files: IndexMap<PathArc, IndexMap<Name, raw::ArtifactRaw>> = IndexMap::new();
    for art in project.artifacts.values() {
        let art = ArtifactIm::from(art.clone());
        let (file, name, raw) = art.into_raw();
        let entry = files.entry(file).or_insert_with(IndexMap::new);
        entry.insert(name, raw);
    }

    let recv_lint = {
        let (send_lint, recv_lint) = ch::bounded(128);
        let (send_arts, recv_arts) = ch::bounded(128);

        for _ in 0..num_cpus::get() {
            take!(=send_lint, =recv_arts);
            spawn(move || {
                for (path, arts) in recv_arts {
                    let path: PathArc = path;
                    macro_rules! handle_err {
                        [$result:expr] => {
                            match $result {
                                Ok(v) => v,
                                Err(err) => {
                                    let l = lint::Lint {
                                        level: lint::Level::Error,
                                        path: Some(path.to_string()),
                                        line: None,
                                        category: lint::Category::SaveProject,
                                        msg: err.to_string(),
                                    };
                                    ch!(send_lint <- l);
                                    continue;
                                }
                            }
                        };
                    }

                    let file = handle_err!(PathFile::create(&path));
                    let mut arts: IndexMap<Name, raw::ArtifactRaw> = arts;
                    arts.sort_keys();
                    let text = match raw::ArtFileType::from_path(&file) {
                        Some(raw::ArtFileType::Toml) => expect!(toml::to_string(&arts)),
                        Some(raw::ArtFileType::Md) => raw::to_markdown(&arts),
                        Some(raw::ArtFileType::Json) => expect!(json::to_string(&arts)),
                        None => unreachable!(),
                    };
                    handle_err!(file.write_str(&text));
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
