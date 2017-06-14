use dev_prefix::*;
use jsonrpc_core::{RpcMethodSync, Params, Error as RpcError};
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

/// `UpdateArtifacts` API Handler
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

pub(in api) fn update_project(data_artifacts: &[ArtifactData],
                              project: &Project,
                              new_artifacts: &[ArtifactData],
                              for_create: bool)
                              -> result::Result<Project, RpcError> {

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
        let (n, a) = match utils::convert_artifact(&project.origin, art_data) {
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
        save_artifacts.insert(n, a);
    }

    // process the new set of artifacts to make sure they are valid
    let mut new_project = Project {
        artifacts: save_artifacts,
        ..project.clone()
    };

    if let Err(err) = user::process_project(&mut new_project) {
        return Err(RpcError {
                       code: constants::SERVER_ERROR,
                       message: err.to_string(),
                       data: None,
                   });
    }

    Ok(new_project)
}
