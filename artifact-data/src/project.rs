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
//! The major exported type and function for loading artifacts.

use time;

use dev_prelude::*;
use artifact;
use implemented;
use lint;
use name::{Name, SubName};
use raw;
use settings;

#[derive(Debug, PartialEq)]
pub struct Project {
    pub paths: Arc<settings::ProjectPaths>,
    pub code_impls: OrderMap<Name, implemented::ImplCode>,
    pub artifacts: OrderMap<Name, artifact::Artifact>,
}

lazy_static!{
    /// Name reference that can exist in source code
    static ref TEXT_REF_RE: Regex = Regex::new(
        &format!(r#"(?xi)
        \[\[(               # start main section
        (?:REQ|SPC|TST)     # all types are supported
        -(?:[{0}]+-)*       # any number of first element
        (?:[{0}]+)          # required end element
        )                   # end main section
        (\.[{0}]+)?         # (optional) sub section
        \]\]                # close text reference
        "#, NAME_VALID_CHARS!())).unwrap();
}

impl Project {
    /// Recursively sort all the items in the project.
    pub fn sort(&mut self) {
        self.code_impls.sort_keys();
        for (_, code) in self.code_impls.iter_mut() {
            code.secondary.sort_keys();
        }
        self.artifacts.sort_keys();
        for (_, art) in self.artifacts.iter_mut() {
            art.sort();
        }
    }

    /// #SPC-read-lint
    ///
    /// TODO WARN:
    /// - references in text that do not exist
    /// - (optional?) poorly formed references in text
    pub fn lint(&self) -> lint::Categorized {
        let (send, recv) = ch::unbounded();

        self.lint_errors(&send);
        self.lint_other(&send);

        drop(send);
        let mut lints = lint::Categorized::default();
        lints.categorize(recv.into_iter());
        lints.sort();
        lints
    }

    /// Lint against only "fatal" errors.
    pub fn lint_errors(&self, send: &Sender<lint::Lint>) {
        lint_partof_dne(send, self);
        lint_partof_types(send, self);
        lint_artifact_text(send, self);
        lint_artifact_done_subnames(send, self);
    }

    /// Lint against non-fatal errors.
    pub fn lint_other(&self, send: &Sender<lint::Lint>) {
        lint_artifact_text_refs(send, self);
        lint_code_impls(send, self);
    }
}

/// Load the project from the given path.
pub fn read_project<P: AsRef<Path>>(
    project_path: P,
) -> result::Result<(lint::Categorized, Project), lint::Categorized> {
    let start_load = time::get_time();
    let mut lints = lint::Categorized::default();

    let paths = {
        let (mut load_lints, paths) = settings::load_project_paths(project_path);
        lints.categorize(load_lints.drain(..));
        if !lints.error.is_empty() {
            lints.sort();
            return Err(lints);
        }

        let mut paths = paths.expect("No lints but also no settings file!");
        paths.code_paths.sort();
        paths.exclude_code_paths.sort();
        paths.artifact_paths.sort();
        paths.exclude_artifact_paths.sort();
        Arc::new(paths)
    };

    let (lint_handle, locs_handle, loaded_handle) = {
        let (send_err, recv_err) = ch::bounded(128);
        let lint_handle = spawn(move || {
            lints.categorize(recv_err.iter());
            lints
        });

        // -------- CODE LOCATIONS --------
        take!(=send_err as errs, =paths as cl_paths);
        let (send_code_path, recv_code_path) = ch::bounded(128);
        spawn(move || {
            settings::walk_paths(&send_code_path, &errs, &cl_paths.code_paths, |path| {
                let abs: &PathAbs = path.as_ref();
                !cl_paths.exclude_code_paths.contains(abs)
            })
        });

        let (send_loc, recv_loc) = ch::bounded(128);
        for _ in 0..4 {
            take!(=recv_code_path, =send_loc, =send_err);
            spawn(move || {
                for file in recv_code_path.iter() {
                    implemented::load_locations(&send_err, &file, &send_loc);
                }
            });
        }

        take!(=send_err as errs);
        let locs_handle = spawn(move || {
            let locs: Vec<_> = recv_loc.iter().collect();
            implemented::join_locations(&errs, locs)
        });

        // -------- ARTIFACTS --------
        take!(=send_err as errs, =paths as cl_paths);
        let (send_artifact_paths, recv_artifact_paths) = ch::bounded(128);
        spawn(move || {
            settings::walk_artifact_paths(
                &send_artifact_paths,
                &errs,
                &cl_paths.artifact_paths,
                &cl_paths.exclude_artifact_paths,
            )
        });

        let (send_artifact_im, recv_artifact_im) = ch::bounded(128);
        for _ in 0..num_cpus::get() {
            take!(=recv_artifact_paths, =send_artifact_im, =send_err);
            spawn(move || {
                for file in recv_artifact_paths {
                    raw::load_file(&send_err, &send_artifact_im, &file);
                }
            });
        }

        take!(=send_err as errs);
        let loaded_handle = spawn(move || {
            let artifacts_im: Vec<_> = recv_artifact_im.iter().collect();
            let (defined, raw) = raw::join_artifacts_raw(&errs, artifacts_im);
            let loaded = artifact::finalize_load_artifact(raw);
            (defined, loaded)
        });

        (lint_handle, locs_handle, loaded_handle)
    };

    let mut lints = lint_handle.finish();

    if !lints.error.is_empty() {
        lints.sort();
        return Err(lints);
    }

    let code_impls = locs_handle.finish();
    let (defined, loaded) = loaded_handle.finish();
    let artifacts = artifact::determine_artifacts(loaded, &code_impls, &defined);

    let mut project = Project {
        paths: paths,
        code_impls: code_impls,
        artifacts: artifacts,
    };

    lints.sort();
    project.sort();

    debug!(
        "project load took {:.3} seconds",
        time::get_time() - start_load
    );
    Ok((lints, project))
}

/// #REQ-family.lint_partof_exists
/// Lint against partofs that do not exist but should (ERROR)
pub(crate) fn lint_partof_dne(lints: &Sender<lint::Lint>, project: &Project) {
    for (name, art) in project.artifacts.iter() {
        for pof in art.partof.iter() {
            if !project.artifacts.contains_key(pof) {
                lints
                    .send(lint::Lint {
                        level: lint::Level::Error,
                        path: Some(art.file.clone().into()),
                        line: None,
                        category: lint::Category::Artifact,
                        msg: format!(
                            "{} defines partof={} which does not exist",
                            name.as_str(),
                            pof.as_str()
                        ),
                    })
                    .expect("send lint");
            }
        }
    }
}

/// #REQ-family.lint_types
/// Lint against partof's that have invalid types.
pub(crate) fn lint_partof_types(lints: &Sender<lint::Lint>, project: &Project) {
    use name::Type::{REQ, SPC, TST};
    for (name, art) in project.artifacts.iter() {
        for pof in art.partof.iter() {
            let invalid = match (name.ty, pof.ty) {
                // SPC can not have part REQ
                (REQ, SPC)
                // TST can not have part REQ
                | (REQ, TST)
                // TST can not have part SPC
                | (SPC, TST) => true,
                _ => false,
            };

            if invalid {
                lints
                    .send(lint::Lint {
                        level: lint::Level::Error,
                        path: Some(art.file.clone().into()),
                        line: None,
                        category: lint::Category::Artifact,
                        msg: format!(
                            "{} cannot have `partof` {}: invalid types.",
                            name.as_str(),
                            pof.as_str(),
                        ),
                    })
                    .expect("send lint");
            }
        }
    }
}

/// #SPC-read-artifact.lint_done
/// Lint that done is not defined on an artifact which has subnames.
pub(crate) fn lint_artifact_done_subnames(lints: &Sender<lint::Lint>, project: &Project) {
    for (name, art) in project.artifacts.iter() {
        if art.impl_.is_done() && !art.subnames.is_empty() {
            lints
                .send(lint::Lint {
                    level: lint::Level::Error,
                    path: Some(art.file.clone().into()),
                    line: None,
                    category: lint::Category::Artifact,
                    msg: format!(
                        "{}: subnames are defined when the `done` field is set.",
                        name.as_str()
                    ),
                })
                .expect("send lint");
        }
    }
}

/// Lint against code_impls
pub(crate) fn lint_code_impls(lints: &Sender<lint::Lint>, project: &Project) {
    use implemented::{CodeLoc, Impl};
    let send_lint = |name: &Name, sub: Option<&SubName>, loc: &CodeLoc, msg: &str| {
        lints
            .send(lint::Lint {
                level: lint::Level::Warn,
                path: Some(loc.file.clone().into()),
                line: Some(loc.line),
                category: lint::Category::ImplCode,
                msg: format!("Invalid code impl #{}. {}", name.full(sub), msg),
            })
            .expect("send lint");
    };
    for (name, code_impl) in project.code_impls.iter() {
        if let Some(art) = project.artifacts.get(name) {
            match art.impl_ {
                Impl::Done(_) => {
                    // #SPC-read-impl.lint_done
                    // impls exist for artifact defined as done
                    if let Some(ref loc) = code_impl.primary {
                        send_lint(name, None, loc, "Artifact's done field is set");
                    }
                    for (sub, loc) in code_impl.secondary.iter() {
                        send_lint(name, Some(sub), loc, "Artifact's done field is set");
                    }
                }
                Impl::Code(_) => {
                    for (sub, loc) in code_impl.secondary.iter() {
                        if !art.subnames.contains(sub) {
                            // #SPC-read-impl.lint_exists
                            // subname ref does not exist
                            send_lint(
                                name,
                                Some(sub),
                                loc,
                                &format!(
                                    "Subname [[{}]] does not exist in artifact's text",
                                    sub.as_str()
                                ),
                            );
                        }
                    }
                }
                Impl::NotImpl => {}
            }
        } else {
            // #SPC-read-impl.lint_subname_exists
            // artifact does not exist!
            if let Some(ref loc) = code_impl.primary {
                send_lint(
                    name,
                    None,
                    loc,
                    &format!("Artifact {} does not exist", name.as_str()),
                );
            }
            for (sub, loc) in code_impl.secondary.iter() {
                send_lint(
                    name,
                    Some(sub),
                    loc,
                    &format!("Artifact {} does not exist", name.as_str()),
                );
            }
        }
    }
}

/// #SPC-read-artifact.lint_text
/// Lint against artifact text being structured incorrectly.
pub(crate) fn lint_artifact_text(lints: &Sender<lint::Lint>, project: &Project) {
    let send_lint = |name: &Name, file: &PathArc, msg: &str| {
        lints
            .send(lint::Lint {
                level: lint::Level::Error,
                path: Some(file.clone().into()),
                line: None,
                category: lint::Category::Artifact,
                msg: format!("{} text is invalid: {}", name.as_str(), msg),
            })
            .expect("send lint");
    };
    for (name, art) in project.artifacts.iter() {
        for line in art.text.lines() {
            if raw::NAME_LINE_RE.is_match(line) {
                send_lint(
                    name,
                    &art.file,
                    "Cannot have a line of the form \"# ART-name\" as that specifies a new \
                     artifact in the markdown format.",
                )
            } else if raw::ATTRS_END_RE.is_match(line) {
                send_lint(
                    name,
                    &art.file,
                    "Cannot have a line of the form \"###+\" as that specifies \
                     the end of the metadata in the markdown format.",
                )
            }
        }
    }
}

/// #SPC-read-artifact.lint_text_refs
/// Lint warnings against invalid references in the artifact text.
pub(crate) fn lint_artifact_text_refs(lints: &Sender<lint::Lint>, project: &Project) {
    let send_lint = |name: &Name, ref_name: &Name, ref_sub: Option<&SubName>, file: &PathArc| {
        lints
            .send(lint::Lint {
                level: lint::Level::Warn,
                path: Some(file.clone().into()),
                line: None,
                category: lint::Category::Artifact,
                msg: format!(
                    "{} has soft reference [[{}]] which does not exist.",
                    name.as_str(),
                    ref_name.full(ref_sub)
                ),
            })
            .expect("send lint");
    };
    for (name, art) in project.artifacts.iter() {
        for captures in TEXT_REF_RE.captures_iter(&art.text) {
            // unwrap: group 1 always exists in regex
            let name_mat = captures.get(1).unwrap();
            // unwrap: pre-validated by regex
            let ref_name = Name::from_str(name_mat.as_str()).unwrap();
            // subname is optional
            let ref_sub = match captures.get(2) {
                Some(sub_mat) => Some(SubName::new_unchecked(sub_mat.as_str())),
                None => None,
            };
            match (project.artifacts.get(&ref_name), &ref_sub) {
                (None, _) => {
                    // specified an artifact that does not exist
                    send_lint(name, &ref_name, ref_sub.as_ref(), &art.file);
                }
                (Some(ref_art), &Some(ref sub)) => {
                    if !ref_art.subnames.contains(sub) {
                        // specified a sub that does not exist
                        send_lint(name, &ref_name, Some(sub), &art.file);
                    }
                }
                _ => {}
            }
        }
    }
}
