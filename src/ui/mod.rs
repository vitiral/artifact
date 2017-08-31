//! This is the cross-ui module
mod types;
pub mod fmt;
mod search;

pub use ui::types::{FmtArtifact, FmtSettings, PercentSearch, SearchSettings};
pub use ui::search::show_artifact;
pub use ui::fmt::{fmt_artifact, fmt_names};
