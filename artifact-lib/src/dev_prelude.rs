pub use std::result;
pub use ergo_std::*;
pub use ergo_config::*;
pub use path_abs::*;
pub use indexmap::*;
pub use failure::*;
pub use expect_macro::*;
pub use failure::Error;

pub type Result<V> = result::Result<V, Error>;
