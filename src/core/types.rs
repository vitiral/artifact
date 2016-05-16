
use std::path;
use std::fmt;
use std::error;
use std::option::Option;
use std::collections::HashMap;

pub type LoadResult<T> = Result<T, LoadError>;
pub type Artifacts = HashMap<ArtName, Artifact>;

#[derive(Debug)]
/// LOC-core-artifacts-enum:<valid artifact types>
pub enum ArtTypes {
    REQ,
    SPC,
    RSK,
    TST,
    LOC,
}

/// Location data type
#[derive(Debug)]
pub struct Loc {
    path: path::PathBuf,
    loc: String,
}

/// LOC-core-artifact-name:<storage of the artifact's name>
#[derive(Debug, PartialEq, Hash)]
pub struct ArtName {
    prefixes: Vec<String>,
}

/// LOC-core-artifact:<artifact definition>
/// The Artifact type. This encapsulates
/// REQ, SPC, RSK, and TST artifacts and
/// contains space to link them
#[derive(Debug)]
pub struct Artifact {
    // directly loaded types
    pub ty: ArtTypes,
    pub name: ArtName,
    pub path: path::PathBuf,
    pub text: String,
    pub partof: Vec<ArtName>,
    pub parts: Vec<ArtName>,
    pub loc: Loc,
    pub done: bool,
    pub ignore: bool,
}

// #[derive(Debug)]
// pub struct Settings {
//     pub disabled: bool,
//     pub paths: Vec<PathBuf>,
//     pub repo_names: HashSet<String>,
// }

/// Error for parsing files into artifacts
/// LOC-core-load-error: <load file error>
#[derive(Debug)]
pub struct LoadError {
    pub desc: String,
}

impl LoadError {
    pub fn new(desc: String) -> LoadError {
        LoadError { desc: desc }
    }
}


impl fmt::Display for LoadError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "ParseError({:?})", self.desc)
    }
}

impl error::Error for LoadError {
    fn description(&self) -> &str {
        "error loading .rsk file"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
