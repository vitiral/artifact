
use dev_prefix::*;
use jsonrpc_core::{IoHandler, SyncMethodCommand, Params, Value, Error as RpcError, ErrorCode};
use serde_json;

use core::prefix::*;
use core;

use super::{ARTIFACTS, PROJECT};

const X_IDS_NOT_FOUND: &'static str = "xIdsNotFound";
const X_INVALID_NAME: &'static str = "xInvalidName";
const X_FILES_NOT_FOUND: &'static str = "xFilesNotFound";
const X_PROCESS_ERROR: &'static str = "xProcessError";
const X_MULTIPLE_ERRORS: &'static str = "xMultipleErrors";
const X_CODE: i64 = -32000;

const SERVER_ERROR: ErrorCode = ErrorCode::ServerError(X_CODE);

lazy_static! {
    pub static ref RPC_HANDLER: IoHandler = init_rpc_handler();
}

fn init_rpc_handler() -> IoHandler {
    let handler = IoHandler::new();
    handler.add_method("GetArtifacts", GetArtifacts);
    handler
}

/// `GetArtifacts` API Handler
struct GetArtifacts;
impl SyncMethodCommand for GetArtifacts {
    fn execute(&self, _: Params) -> result::Result<Value, RpcError> {
        info!("GetArtifacts called");
        let locked = ARTIFACTS.lock().unwrap();
        let artifacts: &Vec<ArtifactData> = locked.as_ref();
        let value = serde_json::to_value(artifacts);
        Ok(value)
    }
}

// helper methods for UpdateArtifacts

fn invalid_params(desc: &str) -> RpcError {
    RpcError {
        code: ErrorCode::InvalidParams,
        message: desc.to_string(),
        data: None,
    }
}

fn parse_error(desc: &str) -> RpcError {
    RpcError {
        code: ErrorCode::ParseError,
        message: desc.to_string(),
        data: None,
    }
}

/// convert an artifact from it's data representation
/// to it's internal artifact representation
fn convert_artifact(artifact_data: &ArtifactData) 
    -> result::Result<(ArtNameRc, Artifact), String> 
{
    Artifact::from_data(&artifact_data).map_err(|err| err.to_string())
}

 /// UpdateArtifacts Handler
struct UpdateArtifacts;
impl SyncMethodCommand for UpdateArtifacts {
    fn execute(&self, params: Params) -> result::Result<Value, RpcError> {
        info!("* UpdateArtifacts");

        let new_artifacts = match params {
            Params::Map(dict) => match dict.get("artifacts") {
                Some(value) => match serde_json::from_value::<Vec<ArtifactData>>(value.clone()) {
                    Ok(a) => a,
                    Err(e) => return Err(parse_error(&format!("{}", e))),
                },
                None => return Err(invalid_params("missing 'artifacts' param")),
            },
            _ => return Err(invalid_params("params must have 'artifacts' key")),
        };

        let mut artifacts = ARTIFACTS.lock().unwrap();
        let project = PROJECT.lock().unwrap();

        let mut map_artifacts: HashMap<u64, ArtifactData> = artifacts.iter_mut()
            .map(|a| (a.id, a.clone())).collect();

        let mut save_artifacts: Artifacts = Artifacts::new();
        // insert the new artifacts and check errors first
        let mut files_not_found: Vec<PathBuf> = Vec::new();
        let mut ids_not_found: Vec<u64> = Vec::new();
        let mut name_errors: Vec<String> = Vec::new();

        for new_artifact in new_artifacts {
            let path = PathBuf::from(&new_artifact.path);
            if !project.files.contains(&path) {
                files_not_found.push(path);
            }
            // remove artifact ids that are getting updated
            if map_artifacts.remove(&new_artifact.id).is_none() {
                ids_not_found.push(new_artifact.id);
            }
            let (n, a) = match convert_artifact(&new_artifact) {
                Ok(v) => v,
                Err(err) => {
                    name_errors.push(err);
                    continue;
                },
            };
            save_artifacts.insert(n, a);
        }

        // craft the error based on all the errors
        let mut data: HashMap<&str, Vec<String>> = HashMap::new();
        let mut err = None;
        if !name_errors.is_empty() {
            data.insert("nameErrors", name_errors);
            err = Some(X_INVALID_NAME);
        } 
        if !files_not_found.is_empty() {
            data.insert("filesNotFound", files_not_found.iter()
                .map(|f| format!("{}", f.display())).collect());
            err = Some(X_FILES_NOT_FOUND);
        } 
        if !ids_not_found.is_empty() {
            data.insert("idsNotFound", ids_not_found.iter()
                .map(u64::to_string).collect());
            err = Some(X_IDS_NOT_FOUND);
        }
        if data.len() > 1 {
            err = Some(X_MULTIPLE_ERRORS);
        }
        if let Some(msg) = err {
            return Err(RpcError {
                code: SERVER_ERROR,
                message: msg.to_string(),
                data: Some(serde_json::to_value(data)),
            });
        }

        for art_data in map_artifacts.values() {
            let (n, a) = match convert_artifact(art_data) {
                Ok(v) => v,
                Err(msg) => return Err(RpcError {
                    code: SERVER_ERROR,
                    message: format!(
                        "Could not convert artifact back {:?}, GOT ERROR: {}",
                        art_data, msg),
                    data: None,
                }),
            };
            save_artifacts.insert(n, a);
        }

        // process the new set of artifacts to make sure they are valid
        let mut new_project = project.clone();
        new_project.artifacts = save_artifacts;

        if let Err(err) = core::process_project(&mut new_project) {
            return Err(RpcError {
                code: SERVER_ERROR,
                message: err.to_string(),
                data: None,
            });
        }

        // TODO: 
        // save artifacts to files and then overwrite *artifacts
        // with the new values
        
        
        // Finally, return ALL the artifacts (the client's cache is outdated)
        return Ok(serde_json::to_value("true"));
    //    let id = match parse_id(req) {
    //        Ok(id) => id,
    //        Err(e) => {
    //            res.set(StatusCode::NotFound);
    //            return res.send(e);
    //        },
    //    };
    //    let mut locked = ARTIFACTS.lock().unwrap();
    //    let artifact = match get_artifact(locked.as_mut(), id) {
    //        Ok(a) => a,
    //        Err(e) => {
    //            res.set(StatusCode::NotFound);
    //            return res.send(e);
    //        },
    //    };
    //    let new = match req.json_as::<Artifact>() {
    //        Ok(a) => a,
    //        Err(e) => {
    //            res.set(StatusCode::BadRequest);
    //            return res.send(format!("{}", e));
    //        },
    //    };
    //    if new.id != id {
    //        res.set(StatusCode::BadRequest);
    //        return res.send("cannot change artifact's id");
    //    }
    //    if new == *artifact {
    //        res.set(StatusCode::NotModified);
    //        return res.send("not modified");
    //    }
    //    *artifact = new;
    //    let data = json::as_pretty_json(artifact);
    //    let str_data = format!("{}", data);
    //    println!("* PUT /artifacts/{} success", id);
    //    config_json_res(&mut res);
    //    res.send(str_data)
    }
}
