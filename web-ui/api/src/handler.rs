
use std::ops::{Deref, DerefMut};

use serde::de::Deserialize;
use jsonrpc_core::{IoHandler, SyncMethodCommand, Params, Value, Error, ErrorCode};
use rustc_serialize::json;
use serde_json;

use Artifact;
use ARTIFACTS;

lazy_static! {
    pub static ref RPC_HANDLER: IoHandler = init_rpc_handler();
}

fn init_rpc_handler() -> IoHandler {
    let handler = IoHandler::new();
    handler.add_method("GetArtifacts", GetArtifacts);
    handler
}

// helper methods

fn get_artifact<'a>(artifacts: &'a mut Vec<Artifact>, id: u64) -> Result<&'a mut Artifact, String> {
    match artifacts.iter_mut().filter(|p| p.id == id).next() {
        Some(a) => Ok(a),
        None => {
            println!("- id not found: {}", id);
            Err(format!("Artifact {} not found", id))
        },
    }
}


/// GetArtifacts Handler
struct GetArtifacts;
impl SyncMethodCommand for GetArtifacts {
    fn execute(&self, params: Params) -> Result<Value, Error> {
        println!("* GetArtifacts");
        let locked = ARTIFACTS.lock().unwrap();
        let artifacts: &Vec<Artifact> = locked.as_ref();
        let value = serde_json::to_value(artifacts);
        Ok(value)
    }
}

fn parse_error(desc: &str) -> Error {
    Error {
        code: ErrorCode::ParseError,
        message: desc.to_string(),
        data: None,
    }
}

fn invalid_params(desc: &str) -> Error {
    Error {
        code: ErrorCode::InvalidParams,
        message: desc.to_string(),
        data: None,
    }
}

// /// UpdateArtifacts Handler
//struct UpdateArtifacts;
//impl SyncMethodCommand for GetArtifacts {
//    fn execute(&self, params: Params) -> Result<Value, Error> {
//        println!("* UpdateArtifacts");

//        let new = match params {
//            Map(dict) => match dict.get("artifacts") {
//                Some(value) => match serde_json::from_value::Vec<Artifact>(value) {
//                    Ok(a) => a,
//                    Err(e) => return parse_error(format!("{}", e)),
//                }
//                None => return invalid_params("missing 'artifacts' param"),
//            }
//            _ => return invalid_params("params must have 'artifacts' key"),
//        };

//        let id = match parse_id(req) {
//            Ok(id) => id,
//            Err(e) => {
//                res.set(StatusCode::NotFound);
//                return res.send(e);
//            },
//        };
//        let mut locked = ARTIFACTS.lock().unwrap();
//        let artifact = match get_artifact(locked.as_mut(), id) {
//            Ok(a) => a,
//            Err(e) => {
//                res.set(StatusCode::NotFound);
//                return res.send(e);
//            },
//        };
//        let new = match req.json_as::<Artifact>() {
//            Ok(a) => a,
//            Err(e) => {
//                res.set(StatusCode::BadRequest);
//                return res.send(format!("{}", e));
//            },
//        };
//        if new.id != id {
//            res.set(StatusCode::BadRequest);
//            return res.send("cannot change artifact's id");
//        }
//        if new == *artifact {
//            res.set(StatusCode::NotModified);
//            return res.send("not modified");
//        }
//        *artifact = new;
//        let data = json::as_pretty_json(artifact);
//        let str_data = format!("{}", data);
//        println!("* PUT /artifacts/{} success", id);
//        config_json_res(&mut res);
//        res.send(str_data)
//    }
//}
