
use std::path;
use std::fmt;
use std::error;
use std::convert::From;
use std::option::Option;
use std::collections::{HashMap, HashSet};

pub type LoadResult<T> = Result<T, LoadError>;
pub type Artifacts = HashMap<ArtName, Artifact>;
pub type Variables = HashMap<String, String>;

#[derive(Debug, PartialEq)]
enum Align {
    Left,
    Center,
    Right,
    None,
}

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
    loc: ArtName,
    path: path::PathBuf,
}

impl<'a> From<&'a str> for Loc {
    /// return a naive version of the Loc object.
    /// which is not checked for validity
    fn from(s: &'a str) -> Loc {
        let mut loc;
        let mut path;
        let split = s.find(':');
        match split {
            None => {
                loc = s;
                path = "";
            }
            Some(_) => {
                let (l, p) = s.split_at(split.unwrap());
                loc = l;
                path = p;
            }
        }
        Loc {
            loc: ArtName::from(loc),
            path: path::PathBuf::from(path),
        }
    }
}

/// LOC-core-artifact-name:<storage of the artifact's name>
#[derive(Debug, PartialEq, Eq, Hash)]
pub struct ArtName(Vec<String>);

impl<'a> From<&'a str> for ArtName {
    fn from(n: &str) -> ArtName {
        ArtName(n.split("-").map(|s| s.to_string()).collect())
    }
}

impl fmt::Display for ArtName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.join("-"))
    }
}

/// LOC-core-artifact:<artifact definition>
/// The Artifact type. This encapsulates
/// REQ, SPC, RSK, and TST artifacts and
/// contains space to link them
#[derive(Debug)]
pub struct Artifact {
    // directly loaded types
    pub ty: ArtTypes,
    pub path: path::PathBuf,
    pub text: String,
    pub refs: Vec<String>,
    pub partof: HashSet<ArtName>,
    pub parts: HashSet<ArtName>,
    pub loc: Loc,
    pub completed: Option<f32>, // completed percent (calculated)
    pub tested: Option<f32>, // tested percent (calculated)
}

#[derive(Debug)]
pub struct Settings {
    pub disabled: bool,
    pub paths: Vec<path::PathBuf>,
    pub repo_names: HashSet<String>,
}

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
