use jsonrpc_core::IoHandler;

use api::crud;

lazy_static! {
    pub static ref RPC_HANDLER: IoHandler = init_rpc_handler();
}

/// the rpc initializer that implements the API spec
fn init_rpc_handler() -> IoHandler {
    // partof: #SPC-rpc-artifacts
    let mut handler = IoHandler::new();
    handler.add_method("CreateArtifacts", crud::CreateArtifacts);
    handler.add_method("ReadArtifacts", crud::ReadArtifacts);
    handler.add_method("UpdateArtifacts", crud::UpdateArtifacts);
    handler.add_method("DeleteArtifacts", crud::DeleteArtifacts);
    init_tracker(&mut handler);
    handler
}

#[cfg(feature = "tracker")]
fn init_tracker(tracker: &mut IoHandler) {
    ::tracker::init_rpc_handler_tracker(tracker);
}

#[cfg(not(feature = "tracker"))]
fn init_tracker(_: &mut IoHandler) {}
