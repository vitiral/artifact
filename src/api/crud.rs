use dev_prefix::*;
use jsonrpc_core::{RpcMethodSync, Params, Error as RpcError, ErrorCode};
use serde_json;

use types::*;
use export::ArtifactData;
use user;
use api::constants;
use api::utils;
use utils::unique_id;

use super::{ARTIFACTS, PROJECT};

//##################################################
//# Create

/// `CreateArtifacts` API Handler
pub struct CreateArtifacts;
impl RpcMethodSync for CreateArtifacts {
    fn call(&self, params: Params) -> result::Result<serde_json::Value, RpcError> {
        info!("* CreateArtifacts");
        do_cu_call(params, true)
    }
}

//##################################################
//# Read

/// `ReadArtifacts` API Handler
pub struct ReadArtifacts;
impl RpcMethodSync for ReadArtifacts {
    fn call(&self, _: Params) -> result::Result<serde_json::Value, RpcError> {
        info!("ReadArtifacts");
        let locked = ARTIFACTS.lock().unwrap();
        let artifacts: &Vec<ArtifactData> = locked.as_ref();
        Ok(serde_json::to_value(artifacts).expect("serde"))
    }
}

//##################################################
//# Update

/// `UpdateArtifacts` API Handler
pub struct UpdateArtifacts;
impl RpcMethodSync for UpdateArtifacts {
    fn call(&self, params: Params) -> result::Result<serde_json::Value, RpcError> {
        info!("* UpdateArtifacts");
        do_cu_call(params, false)
    }
}

//##################################################
//# Delete

/// `DeleteArtifacts` API Handler
pub struct DeleteArtifacts;
impl RpcMethodSync for DeleteArtifacts {
    fn call(&self, params: Params) -> result::Result<serde_json::Value, RpcError> {
        info!("* UpdateArtifacts");

        // get the ids to delete
        let delete_ids = match params {
            Params::Map(mut dict) => {
                match dict.remove("ids") {
                    Some(value) => {
                        match serde_json::from_value::<HashSet<u64>>(value) {
                            Ok(i) => Ok(i),
                            Err(e) => Err(utils::parse_error(&format!("{}", e))),
                        }
                    }
                    None => Err(utils::invalid_params("missing 'ids' param")),
                }
            }
            _ => Err(utils::invalid_params("missing 'ids' key")),
        }?;

        // lock the artifacts
        let mut data_artifacts = ARTIFACTS.lock().unwrap();
        let mut project = PROJECT.lock().unwrap();

        // get all the data artifacts by id
        // TODO: this should probably be moved and used by all these methods
        let mut remaining: HashMap<u64, ArtifactData> = {
            let mut data_artifacts = utils::convert_to_data(&project);
            let mut out: HashMap<u64, ArtifactData> = HashMap::new();
            for dart in data_artifacts.drain(..) {
                let tmp = dart.id;
                if out.insert(tmp, dart).is_some() {
                    return Err(RpcError {
                        code: ErrorCode::InternalError,
                        message: format!("id exists twice: {}", tmp),
                        data: None,
                    });
                }
            }
            out
        };

        // delete artifacts and validate ids
        let mut invalid_ids: Vec<u64> = Vec::new();
        for id in delete_ids {
            if remaining.remove(&id).is_none() {
                invalid_ids.push(id);
                continue;
            };
        }

        // send error message if one exists
        if !invalid_ids.is_empty() {
            let data = format!("{}: {:?}", constants::X_IDS_NOT_FOUND, invalid_ids);
            return Err(RpcError {
                code: constants::SERVER_ERROR,
                message: constants::X_IDS_NOT_FOUND.to_string(),
                data: Some(serde_json::to_value(data).unwrap()),
            });
        }

        // convert from data back to artifacts and process
        let mut save_artifacts = HashMap::new();
        for d in remaining.values() {
            let (name, a) = utils::from_data(&project.origin, d)?;
            save_artifacts.insert(name, a);
        }

        let mut new_project = Project {
            artifacts: save_artifacts,
            ..project.clone()
        };

        process_project(&mut new_project)?;

        let new_artifacts = utils::convert_to_data(&new_project);
        let out = serde_json::to_value(&new_artifacts).expect("serde");

        // save the new project to disk
        utils::dump_artifacts(&new_project)?;

        // store globals and return
        *project = new_project;
        *data_artifacts = new_artifacts;
        Ok(out)
    }
}


//##################################################
//# Some Helpers

/// The create and update commands have almost identical logic
fn do_cu_call(params: Params, for_create: bool) -> result::Result<serde_json::Value, RpcError> {
    // get the changed artifacts
    let updated_artifacts = utils::get_artifacts(params)?;
    let mut data_artifacts = ARTIFACTS.lock().unwrap();
    let mut project = PROJECT.lock().unwrap();

    // perform the update (but don't mutate global yet)
    let new_project = update_project(&data_artifacts, &project, &updated_artifacts, for_create)?;
    drop(updated_artifacts);

    let new_artifacts = utils::convert_to_data(&new_project);
    let out = serde_json::to_value(&new_artifacts).expect("serde");

    // save the new project to disk
    utils::dump_artifacts(&new_project)?;

    // store globals and return
    *project = new_project;
    *data_artifacts = new_artifacts;
    Ok(out)
}

pub(in api) fn update_project(
    data_artifacts: &[ArtifactData],
    project: &Project,
    new_artifacts: &[ArtifactData],
    for_create: bool,
) -> result::Result<Project, RpcError> {

    let (unchanged_artifacts, mut save_artifacts) =
        utils::split_artifacts(project, data_artifacts, new_artifacts, for_create)?;

    for artifact in save_artifacts.values_mut() {
        if for_create {
            // this is a new artifact, give it a new unique id
            artifact.id = unique_id();
        } else {
            // this is a new revision, simply increment it
            artifact.revision += 1;
        }
    }

    // add artifacts that didn't change to new artifacts
    // We have to do this because the calculated values WILL change
    for art_data in unchanged_artifacts.values() {
        let (n, a) = utils::from_data(&project.origin, art_data)?;
        save_artifacts.insert(n, a);
    }

    // process the new set of artifacts to make sure they are valid
    let mut new_project = Project {
        artifacts: save_artifacts,
        ..project.clone()
    };

    process_project(&mut new_project)?;
    Ok(new_project)
}

/// just convert to Rpc error type
fn process_project(project: &mut Project) -> result::Result<(), RpcError> {
    if let Err(err) = user::process_project(project) {
        Err(RpcError {
            code: constants::SERVER_ERROR,
            message: err.to_string(),
            data: None,
        })
    } else {
        Ok(())
    }
}
