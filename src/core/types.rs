// Export these:
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
pub use std::rc::Rc;

// crates
use regex::Regex;

// for type definitions only
use std::fmt;
use std::error;
use std::option::Option;
use std::ascii::AsciiExt;
use std::hash::{Hash, Hasher};
use std::cmp::{PartialEq, Ord, PartialOrd, Ordering};

// definition of new types
pub type LoadResult<T> = Result<T, LoadError>;
pub type Artifacts = HashMap<ArtNameRc, Artifact>;
pub type ArtNameRc = Rc<ArtName>;
pub type ArtNames = HashSet<ArtNameRc>;

pub type Variables = HashMap<String, String>;

lazy_static!{
    // must start with artifact type, followed by "-", followed by at least 1 valid character
    // cannot end with "-"
    pub static ref ART_VALID: Regex = Regex::new(
        r"(REQ|SPC|RSK|TST|LOC)(-[A-Z0-9_-]*[A-Z0-9_])?\z").unwrap();
    pub static ref PARENT_PATH: PathBuf = PathBuf::from("PARENT");
}

#[derive(Debug, Clone, Eq, PartialEq, Hash)]
pub enum ArtType {
    REQ,
    SPC,
    RSK,
    TST,
}

#[derive(Debug, Clone, Eq, PartialEq)]
pub struct Loc {
    pub path: PathBuf,
    pub line_col: (usize, usize),
}

impl Loc {
    pub fn fake() -> Loc {
        Loc {
            path: Path::new("fake").to_path_buf(),
            line_col: (42, 0),
        }
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}({}:{})", self.path.display(),
                    self.line_col.0, self.line_col.1)
    }
}

/// Definition of an artifact name, with Traits for hashing,
/// displaying, etc
/// partof: #SPC-artifact-name (.1)
#[derive(Default, Clone)]
pub struct ArtName {
    pub raw: String,
    pub value: Vec<String>,
}


impl ArtName {
    /// parse name from string and handle errors
    /// see: SPC-artifact-name.2
    pub fn from_str(s: &str) -> LoadResult<ArtName> {
        let value = s.to_ascii_uppercase().replace(' ', "");
        if !ART_VALID.is_match(&value) {
            return Err(LoadError::new("invalid artifact name: ".to_string() + s));
        }
        let out = ArtName {
            raw: s.to_string(),
            value: value.split('-').map(|s| s.to_string()).collect(),
        };
        try!(out.find_type_maybe()); // ensure the type is valid
        Ok(out)
    }

    fn find_type_maybe(&self) -> LoadResult<ArtType> {
        let ty = self.value.get(0).unwrap();
        match ty.as_str() {
            "REQ" => Ok(ArtType::REQ),
            "SPC" => Ok(ArtType::SPC),
            "RSK" => Ok(ArtType::RSK),
            "TST" => Ok(ArtType::TST),
            _ => {
                Err(LoadError::new("Artifact name is invalid, must start with REQ, SPC, etc. Got: "
                                       .to_string() +
                                   self.raw.as_str()))
            }
        }
    }

    pub fn get_type(&self) -> ArtType {
        self.find_type_maybe().unwrap()
    }

    /// see: SPC-artifact-partof-2
    pub fn parent(&self) -> Option<ArtName> {
        if self.value.len() <= 1 {
            return None;
        }
        let mut value = self.value.clone();
        value.pop().unwrap();
        Some(ArtName{raw: value.join("-"), value: value})
    }

    /// return whether this artifact is the root type
    pub fn is_root(&self) -> bool {
        self.value.len() == 1
    }

    pub fn parent_rc(&self) -> Option<ArtNameRc> {
        match self.parent() {
            Some(p) => Some(Rc::new(p)),
            None => None,
        }
    }

    /// see: SPC-artifact-partof-1
    pub fn named_partofs(&self) -> Vec<ArtName> {
        if self.value.len() <= 1 {
            return vec![];
        }
        let ty = self.get_type();
        match ty {
            ArtType::TST => vec![self._get_named_partof("SPC")],
            ArtType::SPC => vec![self._get_named_partof("REQ")],
            ArtType::RSK => vec![],
            ArtType::REQ => vec![],
        }
    }

    /// CAN PANIC
    fn _get_named_partof(&self, ty: &str) -> ArtName {
        let s = ty.to_string() + self.raw.split_at(3).1;
        ArtName::from_str(&s).unwrap()
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
        write!(f, "{}", self.value.join("-"))
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



/// subfunction to parse names from a names-str recusively
/// see: REQ-2-names
fn _parse_names<I>(raw: &mut I, in_brackets: bool) -> LoadResult<Vec<String>>
    where I: Iterator<Item = char>
{
    // hello-[there, you-[are, great]]
    // hello-there, hello-you-are, hello-you-great
    let mut strout = String::new();
    let mut current = String::new();
    loop {
        // SPC-names.1: read one char at a time
        let c = match raw.next() {
            Some(c) => c,
            None => {
                if in_brackets {
                    // SPC-names.2: do validation
                    return Err(LoadError::new("brackets are not closed".to_string()));
                }
                break;
            }
        };
        match c {
            ' ' | '\n' | '\r' => {}, // ignore whitespace
            '[' => {
                if current == "" {
                    // SPC-names.2: more validation
                    return Err(LoadError::new("cannot have '[' after characters ',' or ']' \
                                               or at start of string".to_string()));
                }
                // SPC-names.3: recurse for brackets
                for p in try!(_parse_names(raw, true)) {
                    strout.write_str(&current).unwrap();
                    strout.write_str(&p).unwrap();
                    strout.push(',');
                }
                current.clear();
            }
            ']' => break,
            ',' => {
                strout.write_str(&current).unwrap();
                strout.push(',');
                current.clear();
            }
            _ => current.push(c),
        }
    }
    strout.write_str(&current).unwrap();
    Ok(strout.split(',').filter(|s| s != &"").map(|s| s.to_string()).collect())
}

#[test]
// partof: #TST-names
fn test_parse_names() {
    assert_eq!(_parse_names(&mut "hi, ho".chars(), false).unwrap(), ["hi", "ho"]);
    assert_eq!(_parse_names(&mut "hi-[ho, he]".chars(), false).unwrap(), ["hi-ho", "hi-he"]);
    assert_eq!(_parse_names(
        &mut "hi-[ho, he], he-[ho, hi, ha-[ha, he]]".chars(), false).unwrap(),
               ["hi-ho", "hi-he", "he-ho", "he-hi", "he-ha-ha", "he-ha-he"]);
    assert!(_parse_names(&mut "[]".chars(), false).is_err());
    assert!(_parse_names(&mut "[hi]".chars(), false).is_err());
    assert!(_parse_names(&mut "hi-[ho, [he]]".chars(), false).is_err());
    assert!(_parse_names(&mut "hi-[ho, he".chars(), false).is_err());
}


pub trait LoadFromStr: Sized {
    fn from_str(s: &str) -> LoadResult<Self>;
}

impl LoadFromStr for ArtNameRc {
    fn from_str(s: &str) -> LoadResult<ArtNameRc> {
        Ok(Rc::new(try!(ArtName::from_str(s))))
    }
}

impl LoadFromStr for ArtNames {
    /// Parse a "names str" and convert into a Set of ArtNames
    /// partof: #SPC-names
    fn from_str(partof_str: &str) -> LoadResult<ArtNames> {
        let strs = try!(_parse_names(&mut partof_str.chars(), false));
        let mut out = HashSet::new();
        for s in strs {
            out.insert(Rc::new(try!(ArtName::from_str(&s))));
        }
        Ok(out)
    }
}



/// The Artifact type. This encapsulates
/// REQ, SPC, RSK, and TST artifacts and
/// contains space to link them
#[derive(Clone, Debug)]
pub struct Artifact {
    // directly loaded types
    pub ty: ArtType,
    pub path: PathBuf,
    pub text: String,
    pub partof: ArtNames,
    pub parts: ArtNames,
    pub loc: Option<Loc>,
    pub completed: f32, // completed ratio (calculated)
    pub tested: f32, // tested ratio (calculated)
}

impl Artifact {
    pub fn is_parent(&self) -> bool {
        self.path == PARENT_PATH.as_path()
    }
}

#[derive(Debug, Default, Clone)]
pub struct Settings {
    pub disabled: bool,
    pub paths: VecDeque<PathBuf>,
    pub code_paths: VecDeque<PathBuf>,
    pub exclude_code_paths: VecDeque<PathBuf>,
    pub color: bool,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            disabled: false,
            paths: VecDeque::new(),
            code_paths: VecDeque::new(),
            exclude_code_paths: VecDeque::new(),
            color: true,
        }
    }
}

/// Error for parsing files into artifacts
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
        "error loading rst file"
    }

    fn cause(&self) -> Option<&error::Error> {
        None
    }
}
