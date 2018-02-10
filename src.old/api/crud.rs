use dev_prefix::*;
use jsonrpc_core::{Error as RpcError, ErrorCode, Params, RpcMethodSync};
use serde_json;

use types::*;
use export::ArtifactData;
use user;
use api::constants;
use api::utils;
use utils::unique_id;

use super::LOCKED;

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
        let locked = LOCKED.lock().unwrap();
        Ok(serde_json::to_value(&locked.project_data).expect("serde"))
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
        info!("* DeleteArtifacts");
        // lock the artifacts
        let mut locked = LOCKED.lock().unwrap();
        if locked.cmd.readonly {
            return Err(utils::readonly_error());
        }

        // get the ids to delete
        let delete_ids = match params {
            Params::Map(mut dict) => match dict.remove("ids") {
                Some(value) => match serde_json::from_value::<HashSet<u64>>(value) {
                    Ok(i) => Ok(i),
                    Err(e) => Err(utils::parse_error(&format!("{}", e))),
                },
                None => Err(utils::invalid_params("missing 'ids' param")),
            },
            _ => Err(utils::invalid_params("missing 'ids' key")),
        }?;

        // get all the data artifacts by id
        // TODO: this should probably be moved and used by all these methods
        let mut remaining: HashMap<u64, ArtifactData> = {
            let mut data_artifacts = locked.project_data.artifacts.clone();
            let mut out: HashMap<u64, ArtifactData> = HashMap::new();
            for dart in data_artifacts.drain(..) {
                let id = dart.id;
                if out.insert(id, dart).is_some() {
                    return Err(RpcError {
                        code: ErrorCode::InternalError,
                        message: format!("id exists twice: {}", id),
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
            }
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
            let (name, a) = utils::from_data(&locked.project.origin, d)?;
            save_artifacts.insert(name, a);
        }

        let mut new_project = Project {
            artifacts: save_artifacts,
            ..locked.project.clone()
        };

        process_project(&mut new_project)?;

        let new_project_data = new_project.to_data();
        let out = serde_json::to_value(&new_project_data).expect("serde");

        // save the new project to disk
        utils::dump_artifacts(&new_project)?;

        // store globals and return
        locked.project = new_project;
        locked.project_data = new_project_data;
        Ok(out)
    }
}


//##################################################
//# Some Helpers

/// The create and update commands have almost identical logic
fn do_cu_call(params: Params, for_create: bool) -> result::Result<serde_json::Value, RpcError> {
    // get the changed artifacts
    let mut locked = LOCKED.lock().unwrap();
    if locked.cmd.readonly {
        return Err(utils::readonly_error());
    }

    let updated_artifacts = utils::get_artifacts(params)?;

    // perform the update (but don't mutate global yet)
    let new_project = update_project(
        &locked.project_data.artifacts,
        &locked.project,
        &updated_artifacts,
        for_create,
    )?;
    drop(updated_artifacts);

    let new_project_data = new_project.to_data();
    let out = serde_json::to_value(&new_project_data).expect("serde");

    // save the new project to disk
    utils::dump_artifacts(&new_project)?;

    // store globals and return
    locked.project = new_project;
    locked.project_data = new_project_data;
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
