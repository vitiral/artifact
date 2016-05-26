
use std::path;
use std::fmt;
use std::error;
use std::convert::From;
use std::option::Option;
use std::collections::{HashMap, HashSet, VecDeque};
use std::ascii::AsciiExt;
use std::hash::{Hash, Hasher};
use std::cmp::{PartialEq, Ord, PartialOrd, Ordering};

use regex::Regex;

pub type LoadResult<T> = Result<T, LoadError>;
pub type Artifacts = HashMap<ArtName, Artifact>;

// LOC-core-vars-sruct
pub type Variables = HashMap<String, String>;

lazy_static!{
    // must start with artifact type, followed by "-", followed by at least 1 valid character
    // cannot end with "-"
    pub static ref ART_VALID: Regex = Regex::new(
        r"(REQ|SPC|RSK|TST|LOC)-[A-Z0-9_-]*[A-Z0-9_]\z").unwrap();
}

#[derive(Debug, Clone, Eq, PartialEq)]
/// LOC-core-artifacts-enum:<valid artifact types>
pub enum ArtType {
    REQ,
    SPC,
    RSK,
    TST,
    LOC,
}

/// LOC-core-loc<Location data type>
#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Loc {
    pub loc: ArtName,
    pub path: path::PathBuf,
}

impl Loc {
    /// return Loc
    /// the path is not checked for validity yet
    pub fn from_str(s: &str) -> LoadResult<Loc> {
        let loc;
        let path;
        let split = s.find(':');
        match split {
            None => {
                loc = s;
                path = "";
            }
            Some(split) => {
                let (l, p) = s.split_at(split);
                loc = l;
                let (_, p) = p.split_at(1); // throw away ':'
                path = p.trim();
            }
        }
        Ok(Loc {
            loc: try!(ArtName::from_str(loc)),
            path: path::PathBuf::from(path),
        })
    }

    pub fn valid(&self) -> bool {
        if self.path.exists() {
            // TODO: also check to make sure the name exists in the file
            true
        } else {
            false
        }
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}: {}", self.path.to_string_lossy().as_ref(), self.loc)
    }
}

#[test]
fn test_loc() {
    let result = Loc::from_str("LOC-bar: path/is/cool").unwrap();
    assert_eq!(result.loc, ArtName::from_str("loc-BAR").unwrap());
    assert_eq!(result.path, path::PathBuf::from("path/is/cool"));
}

/// LOC-core-artifact-name:<storage of the artifact's name>
/// also contains logic for finding the artifact's type
/// (as it is based on the name)
// TODO: Hash and Eq have to be defined to ONLY care about
// value. raw is simply for displaying on the ui
#[derive(Clone)]
pub struct ArtName {
    pub raw: String,
    pub value: Vec<String>,
}

impl ArtName {
    /// LOC-find-type:<find a valid type or error>
    fn find_type_maybe(&self) -> LoadResult<ArtType> {
        let ty = self.value.get(0).unwrap();
        match ty.as_str() {
            "REQ" => Ok(ArtType::REQ),
            "SPC" => Ok(ArtType::SPC),
            "RSK" => Ok(ArtType::RSK),
            "TST" => Ok(ArtType::TST),
            "LOC" => Ok(ArtType::LOC),
            _ => {
                Err(LoadError::new("Artifact name is invalid, must start with REQ, SPC, etc:"
                                       .to_string() +
                                   self.raw.as_str()))
            }
        }
    }

    pub fn get_type(&self) -> ArtType {
        return self.find_type_maybe().unwrap();
    }

    pub fn from_str(s: &str) -> LoadResult<ArtName> {
        // REQ-core-artifacts-name: strip spaces, ensure valid chars
        // LOC-name-check:<make sure name is valid>
        let value = s.to_ascii_uppercase().replace(' ', "");
        if !ART_VALID.is_match(&value) {
            return Err(LoadError::new("invalid artifact name: ".to_string() + s));
        }
        let out = ArtName {
            raw: s.to_string(),
            value: value.split("-").map(|s| s.to_string()).collect(),
        };
        try!(out.find_type_maybe()); // ensure the type is valid
        Ok(out)
    }

    pub fn parent(&self) -> Option<ArtName> {
        if self.value.len() <= 1 {
            return None;
        }
        let mut value = self.value.clone();
        value.pop().unwrap();
        Some(ArtName{raw: value.join("-"), value: value})
    }
}

#[test]
fn test_artname_parent() {
    let name = ArtName::from_str("REQ-foo-bar-b").unwrap();
    let parent = name.parent().unwrap();
    assert_eq!(parent, ArtName::from_str("REQ-foo-bar").unwrap());
    let parent = parent.parent().unwrap();
    assert_eq!(parent, ArtName::from_str("REQ-foo").unwrap());
    let parent = parent.parent().unwrap();
    let req = ArtName::from_str("REQ-2").unwrap().parent().unwrap();
    assert_eq!(parent, req);
    assert!(parent.parent().is_none());
}

impl fmt::Display for ArtName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.raw)
    }
}

impl fmt::Debug for ArtName {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{:?}", self.value)
    }
}

impl Hash for ArtName {
    fn hash<H: Hasher>(&self, state: &mut H) {
        self.value.hash(state);
    }
}

impl PartialEq for ArtName {
    fn eq(&self, other: &ArtName) -> bool {
        self.value == other.value
    }

    fn ne(&self, other: &ArtName) -> bool {
        self.value != other.value
    }
}

impl Eq for ArtName {}

impl Ord  for ArtName {
    fn cmp(&self, other: &Self) -> Ordering {
        self.value.cmp(&other.value)
    }
}

impl PartialOrd for ArtName {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        self.value.partial_cmp(&other.value)
    }
}

/// LOC-core-artifact:<artifact definition>
/// The Artifact type. This encapsulates
/// REQ, SPC, RSK, and TST artifacts and
/// contains space to link them
#[derive(Debug)]
pub struct Artifact {
    // directly loaded types
    pub ty: ArtType,
    pub path: path::PathBuf,
    pub text: String,
    pub refs: Vec<String>,
    pub partof: HashSet<ArtName>,
    pub parts: HashSet<ArtName>,
    pub loc: Option<Loc>,
    pub completed: f32, // completed percent (calculated)
    pub tested: f32, // tested percent (calculated)
}

#[derive(Debug)]
/// LOC-core-settings
pub struct Settings {
    pub disabled: bool,
    pub paths: VecDeque<path::PathBuf>,
    pub repo_names: HashSet<String>,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            disabled: false,
            paths: VecDeque::new(),
            repo_names: HashSet::new(),
        }
    }
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
        write!(f, "Parse Errors: {}", self.desc)
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
