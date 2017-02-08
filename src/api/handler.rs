
use dev_prefix::*;
use jsonrpc_core::{IoHandler, RpcMethodSync, Params, Error as RpcError};
use jsonrpc_core;
use serde_json;

use core::prefix::*;

use api::update;

use super::ARTIFACTS;

/// the rpc initializer that implements the API spec
fn init_rpc_handler() -> IoHandler {
    // partof: #SPC-rpc-artifacts
    let mut handler = IoHandler::new();
    handler.add_method("GetArtifacts", GetArtifacts);
    handler.add_method("UpdateArtifacts", update::UpdateArtifacts);
    handler
}

lazy_static! {
    pub static ref RPC_HANDLER: IoHandler = init_rpc_handler();
}


/// `GetArtifacts` API Handler
struct GetArtifacts;
impl RpcMethodSync for GetArtifacts {
    fn call(&self, _: Params) -> result::Result<jsonrpc_core::Value, RpcError> {
        info!("GetArtifacts called");
        let locked = ARTIFACTS.lock().unwrap();
        let artifacts: &Vec<ArtifactData> = locked.as_ref();
        let out = {
            // FIXME: when jsonrpc-core uses serde 0.9
            let value = serde_json::to_value(artifacts).unwrap();
            let s = serde_json::to_string(&value).unwrap();
            jsonrpc_core::Value::from_str(&s).unwrap()
        };
        Ok(out)
    }
}
