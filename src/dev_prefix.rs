//! stdlib and external library prefix for core module
//! TODO: this should be it's own crate.... maybe

// stdlib traits
pub use std::ascii::AsciiExt; // to_ascii_uppercase(), etc
pub use std::io::{Read, Seek, SeekFrom, Write};
pub use std::fmt::{Debug, Write as FmtWrite};
pub use std::iter::FromIterator;
pub use std::clone::Clone;
pub use std::default::Default;
pub use std::convert::AsRef;
pub use std::str::FromStr;
pub use std::iter::Iterator;
pub use std::ops::{Deref, DerefMut};

// stdlib modules
pub use std::env;
pub use std::clone;
pub use std::fs;
pub use std::fmt;
pub use std::error;
pub use std::io;
pub use std::str;
pub use std::result;


// stdlib structs
pub use std::ffi::OsString;
pub use std::path::{Path, PathBuf};
pub use std::collections::{HashMap, HashSet, VecDeque};
pub use std::sync::Arc;

// crates
pub use error_chain::ChainedError;
pub use regex::{Regex, RegexBuilder};
pub use serde::{Deserialize, Serialize};
