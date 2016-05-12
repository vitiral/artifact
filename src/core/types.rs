
use std::path;
use std::fmt;
use std::error;
use std::option::Option;
use std::collections::HashMap;

pub type LoadResult<T> = Result<T, ParseFileError>;
pub type Artifacts = HashMap<String, Artifact>;

#[derive(Debug)]
pub enum ArtTypes {
    REQ,
    SPC,
    RSK,
    TST,
}

/// Location data type
#[derive(Debug)]
pub struct Loc {
    path: path::PathBuf,
    loc: String,
}

/// The Artifact type. This encapsulates
/// REQ, SPC, RSK, and TST artifacts and
/// contains space to link them
///
/// $LOC-struct-Artifact
#[derive(Debug)]
pub struct Artifact {
    // directly loaded types
    pub ty: ArtTypes,
    pub name: String,
    pub path: path::PathBuf,
    pub text: String,
    pub extra: String,
    pub partof_str: String,
    pub loc_str: String,
    pub done: bool,
    pub ignore: bool,

    // calculated types
    pub partof: Vec<Artifact>,
    pub parts: Vec<Artifact>,
    pub loc: Option<Loc>,
}


/// Error for parsing files into artifacts
///
/// $LOC-struct-parse-error
#[derive(Debug)]
pub struct ParseFileError {
    pub desc: String,
}

impl ParseFileError {
    pub fn new(desc: String) -> ParseFileError {
        ParseFileError{desc: desc}
    }
}


impl fmt::Display for ParseFileError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseError({:?})", self.desc)
    }
}

impl error::Error for ParseFileError {
    fn description(&self) -> &str {
        "error parsing file"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
