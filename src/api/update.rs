#![allow(dead_code)]
use dev_prefix::*;
use jsonrpc_core::{RpcMethodSync, Params, Error as RpcError, ErrorCode};
use serde_json;

use types::*;
use export::ArtifactData;
use user;
use api::constants;
use api::utils;

use super::{ARTIFACTS, PROJECT};

/// convert an artifact from it's data representation
/// to it's internal artifact representation
fn convert_artifact(origin: &Path,
                    artifact_data: &ArtifactData)
                    -> result::Result<(NameRc, Artifact), String> {
    Artifact::from_data(origin, artifact_data).map_err(|err| err.to_string())
}

/// pull out the artifacts from the params
fn parse_new_artifacts(params: Params) -> result::Result<Vec<ArtifactData>, RpcError> {
    match params {
        Params::Map(mut dict) => {
            match dict.remove("artifacts") {
                Some(value) => {
                    match serde_json::from_value::<Vec<ArtifactData>>(value) {
                        Ok(a) => Ok(a),
                        Err(e) => Err(utils::parse_error(&format!("{}", e))),
                    }
                }
                None => Err(utils::invalid_params("missing 'artifacts' param")),
            }
        }
        _ => Err(utils::invalid_params("params must have 'artifacts' key")),
    }
}

/// split artifacts into artifacts which are unchanged and
/// artifacts which are changed.
///
/// Also do lots of error checking and validation
#[allow(useless_let_if_seq)]
pub fn split_artifacts(project: &Project,
                       data_artifacts: &[ArtifactData],
                       new_artifacts: &[ArtifactData])
                       -> result::Result<(HashMap<u64, ArtifactData>, Artifacts), RpcError> {
    let mut unchanged_artifacts: HashMap<u64, ArtifactData> =
        data_artifacts.iter().map(|a| (a.id, a.clone())).collect();

    let mut save_artifacts: Artifacts = Artifacts::new();

    // buffer errors to give better error messages
    let mut files_not_found: Vec<PathBuf> = Vec::new();
    let mut ids_not_found: Vec<u64> = Vec::new();
    let mut name_errors: Vec<String> = Vec::new();

    for new_artifact in new_artifacts {
        let path = project.origin.join(&new_artifact.path);
        if !project.files.contains(&path) {
            files_not_found.push(path);
        }
        // remove artifact ids that are getting updated
        if unchanged_artifacts.remove(&new_artifact.id).is_none() {
            ids_not_found.push(new_artifact.id);
        }
        let (n, a) = match convert_artifact(&project.origin, new_artifact) {
            Ok(v) => v,
            Err(err) => {
                name_errors.push(err);
                continue;
            }
        };
        save_artifacts.insert(n, a);
    }

    // craft the error based on all the errors
    let mut data: HashMap<&str, Vec<String>> = HashMap::new();
    let mut err = None;
    if !name_errors.is_empty() {
        data.insert("nameErrors", name_errors);
        err = Some(constants::X_INVALID_NAME);
    }
    if !files_not_found.is_empty() {
        data.insert("filesNotFound",
                    files_not_found.iter().map(|f| format!("{}", f.display())).collect());
        err = Some(constants::X_FILES_NOT_FOUND);
    }
    if !ids_not_found.is_empty() {
        data.insert("idsNotFound",
                    ids_not_found.iter().map(u64::to_string).collect());
        err = Some(constants::X_IDS_NOT_FOUND);
    }
    if data.len() > 1 {
        err = Some(constants::X_MULTIPLE_ERRORS);
    }
    if let Some(msg) = err {
        return Err(RpcError {
                       code: constants::SERVER_ERROR,
                       message: msg.to_string(),
                       data: Some(serde_json::to_value(data).unwrap()),
                   });
    }

    Ok((unchanged_artifacts, save_artifacts))
}


/// Update artifacts with new ones
pub fn update_artifacts(data_artifacts: &[ArtifactData],
                        project: &Project,
                        new_artifacts: &[ArtifactData])
                        -> result::Result<Project, RpcError> {

    let (unchanged_artifacts, mut save_artifacts) =
        split_artifacts(project, data_artifacts, new_artifacts)?;

    // add artifacts that didn't change to new artifacts
    for art_data in unchanged_artifacts.values() {
        let (n, a) = match convert_artifact(&project.origin, art_data) {
            Ok(v) => v,
            Err(msg) => {
                let e = RpcError {
                    code: constants::SERVER_ERROR,
                    message: format!("Could not convert artifact back {:?}, GOT ERROR: {}",
                                     art_data,
                                     msg),
                    data: None,
                };
                return Err(e);
            }
        };
        // TODO: preserve old id here?
        save_artifacts.insert(n, a);
    }

    // process the new set of artifacts to make sure they are valid
    let mut new_project = Project { artifacts: save_artifacts, ..project.clone() };

    if let Err(err) = user::process_project(&mut new_project) {
        return Err(RpcError {
                       code: constants::SERVER_ERROR,
                       message: err.to_string(),
                       data: None,
                   });
    }

    Ok(new_project)
}

/// `UpdateArtifacts` Handler
pub struct UpdateArtifacts;
impl RpcMethodSync for UpdateArtifacts {
    fn call(&self, params: Params) -> result::Result<serde_json::Value, RpcError> {
        info!("* UpdateArtifacts");

        // get the changed artifacts
        let updated_artifacts = parse_new_artifacts(params)?;
        let mut data_artifacts = ARTIFACTS.lock().unwrap();
        let mut project = PROJECT.lock().unwrap();

        // perform the update
        let new_project = update_artifacts(&data_artifacts, &project, &updated_artifacts)?;

        drop(updated_artifacts);

        // get the ProjectText
        let text = match user::ProjectText::from_project(&new_project) {
            Ok(t) => t,
            Err(e) => {
                return Err(RpcError {
                               code: ErrorCode::InternalError,
                               message: format!("{:?}", e.display()),
                               data: None,
                           })
            }
        };

        // save the ProjectText to files
        if let Err(e) = text.dump() {
            return Err(RpcError {
                           code: ErrorCode::InternalError,
                           message: format!("{:?}", e.display()),
                           data: None,
                       });
        }

        // get the data artifacts
        let new_artifacts: Vec<_> = new_project.artifacts
            .iter()
            .map(|(n, a)| a.to_data(&project.origin, n))
            .collect();

        let out = serde_json::to_value(&new_artifacts).expect("serde");
        // store globals and return
        *project = new_project;
        *data_artifacts = new_artifacts;
        Ok(out)
    }
}
