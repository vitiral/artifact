use dev_prefix::*;
use jsonrpc_core::{Error as RpcError, ErrorCode, Params};
use serde_json;

use types::*;
use export::ArtifactData;
use user;
use api::constants;

pub fn invalid_params(desc: &str) -> RpcError {
    RpcError {
        code: ErrorCode::InvalidParams,
        message: desc.to_string(),
        data: None,
    }
}

pub fn parse_error(desc: &str) -> RpcError {
    RpcError {
        code: ErrorCode::ParseError,
        message: desc.to_string(),
        data: None,
    }
}

pub fn readonly_error() -> RpcError {
    RpcError {
        code: ErrorCode::MethodNotFound,
        message: "method not available when readonly=true".to_string(),
        data: None,
    }
}

/// convert an artifact from it's data representation
/// to it's internal artifact representation
pub fn convert_artifact(
    origin: &Path,
    artifact_data: &ArtifactData,
) -> result::Result<(NameRc, Artifact), String> {
    Artifact::from_data(origin, artifact_data).map_err(|err| err.to_string())
}

/// pull out the artifacts from the params
pub fn get_artifacts(params: Params) -> result::Result<Vec<ArtifactData>, RpcError> {
    match params {
        Params::Map(mut dict) => match dict.remove("artifacts") {
            Some(value) => match serde_json::from_value::<Vec<ArtifactData>>(value) {
                Ok(a) => Ok(a),
                Err(e) => Err(parse_error(&format!("{}", e))),
            },
            None => Err(invalid_params("missing 'artifacts' param")),
        },
        _ => Err(invalid_params("params must have 'artifacts' key")),
    }
}

pub fn from_data(
    origin: &Path,
    data: &ArtifactData,
) -> result::Result<(NameRc, Artifact), RpcError> {
    match convert_artifact(origin, data) {
        Ok(v) => Ok(v),
        Err(msg) => {
            let e = RpcError {
                code: constants::SERVER_ERROR,
                message: format!(
                    "Could not convert artifact back {:?}, GOT ERROR: {}",
                    data,
                    msg
                ),
                data: None,
            };
            Err(e)
        }
    }
}

pub fn dump_artifacts(project: &Project) -> result::Result<(), RpcError> {
    // get the raw ProjectText for saving to disk
    let text = match user::ProjectText::from_project(project) {
        Ok(t) => t,
        Err(e) => {
            return Err(RpcError {
                code: ErrorCode::InternalError,
                message: format!("{}", e.display()),
                data: None,
            })
        }
    };

    // save the ProjectText to files
    if let Err(e) = text.dump() {
        return Err(RpcError {
            code: ErrorCode::InternalError,
            message: format!("{}", e.display()),
            data: None,
        });
    }
    Ok(())
}

/// split artifacts into artifacts which are unchanged and
/// artifacts which are changed.
///
/// Also do lots of error checking and validation
///
/// If create flag is true, this is for the Create command
/// (only expect new artifacts). Otherwise it is for the
/// Update command (only expect artifacts that exist).
#[allow(useless_let_if_seq)]
pub fn split_artifacts(
    project: &Project,
    data_artifacts: &[ArtifactData],
    new_artifacts: &[ArtifactData],
    for_create: bool,
) -> result::Result<(HashMap<u64, ArtifactData>, Artifacts), RpcError> {
    let mut unchanged_artifacts: HashMap<u64, ArtifactData> =
        data_artifacts.iter().map(|a| (a.id, a.clone())).collect();

    let mut save_artifacts: Artifacts = Artifacts::new();

    // buffer errors to give better error messages
    let mut files_not_found: Vec<PathBuf> = Vec::new();
    let mut invalid_ids: Vec<u64> = Vec::new();
    let mut invalid_revisions: Vec<u64> = Vec::new();
    let mut name_errors: Vec<String> = Vec::new();
    let mut name_overlap: Vec<String> = Vec::new();

    for new_artifact in new_artifacts {
        let path = project.origin.join(&new_artifact.def);
        if !project.files.contains(&path) {
            files_not_found.push(path);
        }
        if for_create {
            // when creating, id and revision must == 0
            if new_artifact.id != 0 {
                invalid_ids.push(new_artifact.id)
            }
            if new_artifact.revision != 0 {
                invalid_revisions.push(new_artifact.revision)
            }
        } else if let Some(a) = unchanged_artifacts.remove(&new_artifact.id) {
            // Artifact exists but revision must also be identical.
            // This ensures that the artifact didn't "change out from under" the user.
            if new_artifact.revision != a.revision {
                invalid_revisions.push(new_artifact.revision)
            }
        } else {
            // must update only existing ids
            invalid_ids.push(new_artifact.id)
        }
        let (name, a) = match convert_artifact(&project.origin, new_artifact) {
            Ok(v) => v,
            Err(err) => {
                name_errors.push(err);
                continue;
            }
        };
        if save_artifacts.insert(name.clone(), a).is_some() {
            name_overlap.push(format!("{}", name));
        }
    }

    // check that there are no name collisions
    {
        let mut existing_names: HashSet<NameRc> = HashSet::new();
        for name in unchanged_artifacts
            .values()
            .map(
                |a| NameRc::from_str(&a.name).unwrap(), /* already validated */
            )
            .chain(save_artifacts.keys().cloned())
        {
            if !existing_names.insert(name.clone()) {
                name_overlap.push(format!("{}", name));
            }
        }
    }

    // craft the error based on all the errors
    let mut data: HashMap<&str, Vec<String>> = HashMap::new();
    let mut err = None;
    if !name_errors.is_empty() {
        data.insert(constants::X_INVALID_NAME, name_errors);
        err = Some(constants::X_INVALID_NAME);
    }
    if !files_not_found.is_empty() {
        data.insert(
            constants::X_FILES_NOT_FOUND,
            files_not_found
                .iter()
                .map(|f| format!("{}", f.display()))
                .collect(),
        );
        err = Some(constants::X_FILES_NOT_FOUND);
    }
    if !invalid_ids.is_empty() {
        let id_strs = invalid_ids.iter().map(u64::to_string).collect();
        if for_create {
            data.insert(constants::X_IDS_EXIST, id_strs);
            err = Some(constants::X_IDS_EXIST);
        } else {
            data.insert(constants::X_IDS_NOT_FOUND, id_strs);
            err = Some(constants::X_IDS_NOT_FOUND);
        }
    }
    if !invalid_revisions.is_empty() {
        let revision_strs = invalid_revisions.iter().map(u64::to_string).collect();
        data.insert(constants::X_INVALID_REVISIONS, revision_strs);
        err = Some(constants::X_INVALID_REVISIONS);
    }
    if !name_overlap.is_empty() {
        data.insert(constants::X_NAMES_OVERLAP, name_overlap);
        err = Some(constants::X_NAMES_OVERLAP);
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
