use dev_prefix::*;

use jsonrpc_core::ErrorCode;

pub const X_IDS_EXIST: &'static str = "xIdsExist";
pub const X_IDS_NOT_FOUND: &'static str = "xIdsNotFound";
pub const X_INVALID_REVISIONS: &'static str = "xInvalidRevisions";
pub const X_INVALID_NAME: &'static str = "xInvalidName";
pub const X_NAMES_OVERLAP: &'static str = "xNamesOverlap";
pub const X_FILES_NOT_FOUND: &'static str = "xFilesNotFound";
// const X_PROCESS_ERROR: &'static str = "xProcessError";
pub const X_MULTIPLE_ERRORS: &'static str = "xMultipleErrors";
pub const X_CODE: i64 = -32_000;

pub const SERVER_ERROR: ErrorCode = ErrorCode::ServerError(X_CODE);
