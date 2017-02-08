use jsonrpc_core::{Error as RpcError, ErrorCode};

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
