// Traits
pub use std::io::{Read, Write};
pub use std::fmt::Write as WriteStr;
pub use std::iter::FromIterator;
pub use std::clone::Clone;
pub use std::convert::AsRef;
pub use std::str::FromStr;

// stdlib
pub use std::fs;
pub use std::path::{Path, PathBuf};
pub use std::collections::{HashMap, HashSet, VecDeque};

// crates
pub use strfmt;
pub use regex::Regex;
pub use toml::{Parser, Value, Table};

// modules
pub use core::types::*;
pub use core::vars;
