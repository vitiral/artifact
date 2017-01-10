/*  rst: the requirements tracking tool made for developers
    Copyright (C) 2016  Garrett Berg <@vitiral, vitiral@gmail.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the Lesser GNU General Public License as published 
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the Lesser GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/
//! project wide types
 
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use dev_prefix::*;
pub use super::{ArtifactData, LocData, Text};

pub type Artifacts = HashMap<ArtNameRc, Artifact>;
pub type ArtNameRc = Arc<ArtName>;
pub type ArtNames = HashSet<ArtNameRc>;
pub type Variables = HashMap<String, String>;

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        // no external error chains (yet)
    }

    foreign_links {
        TomlDecode(::toml::DecodeError);
        Io(::std::io::Error);
        Fmt(fmt::Error);
        StrFmt(::strfmt::FmtError);
    }

    errors {
        // Loading errors
        Load(desc: String) {
            description("Misc error while loading artifacts")
            display("Error loading: {}", desc)
        }
        TomlParse(locs: String) {
            description("Error while parsing TOML file")
            display("Error parsing TOML: {}", locs)
        }
        MissingTable {
            description("Must contain a single table")
        }
        InvalidName(desc: String) {
            description("invalid artifact name")
            display("invalid artifact name: \"{}\"", desc)
        }
        InvalidAttr(name: String, attr: String) {
            description("artifact has invalid attribute")
            display("Artifact {} has invalid attribute: {}", name, attr)
        }
        InvalidSettings(desc: String) {
            description("invalid settings")
            display("invalid settings: {}", desc)
        }
        InvalidArtifact(name: String, desc: String) {
            description("invalid artifact")
            display("artifact {} is invalid: {}", name, desc)
        }
        InvalidVariable(desc: String) {
            description("invalid variable")
            display("invalid variable: {}", desc)
        }

        // Processing errors
        InvalidTextVariables {
            description("couldn't resolve some text variables")
        }
        InvalidPartof {
            description("Some artifacts have invalid partof attributes")
        }
        LocNotFound {
            description("errors while finding implementation locations")
        }
        InvalidUnicode(path: String) {
            description("we do not yet support non-unicode paths")
            display("invalid unicode in path: {}", path)
        }

        // Misc errors
        PathNotFound(desc: String) {
            description("invalid path")
            display("Path does not exist: {}", desc)
        }
    }
}

lazy_static!{
    // must start with artifact type, followed by "-", followed by at least 1 valid character
    // cannot end with "-"
    pub static ref ART_VALID: Regex = Regex::new(
        r"^(REQ|SPC|RSK|TST)(-[A-Z0-9_-]*[A-Z0-9_])?$").unwrap();
    pub static ref PARENT_PATH: PathBuf = PathBuf::from("PARENT");
    pub static ref INCREMENTING_ID: AtomicUsize = AtomicUsize::new(0);
}

/// used for artifact ids
fn get_unique_id() -> usize {
    INCREMENTING_ID.fetch_add(1, AtomicOrdering::SeqCst)
}

pub trait LoadFromStr: Sized {
    fn from_str(s: &str) -> Result<Self>;
}

/// represents the results and all the data necessary 
/// to reconstruct a loaded project
#[derive(Debug, Clone)]
pub struct Project {
    pub artifacts: Artifacts,
    pub settings: Settings,
    pub variables: Variables,
    pub files: HashSet<PathBuf>,
    pub dne_locs: HashMap<ArtName, Loc>,

    // preserved locations where each piece is from
    // note: artifacts have path member
    pub settings_map: HashMap<PathBuf, Settings>,
    pub raw_settings_map: HashMap<PathBuf, RawSettings>,
    pub variables_map: HashMap<PathBuf, Variables>,
    pub repo_map: HashMap<PathBuf, PathBuf>,
}

impl Project {
    pub fn new() -> Project {
        Project {
            artifacts: Artifacts::new(),
            settings: Settings::new(),
            variables: Variables::new(),
            files: HashSet::new(),
            dne_locs: HashMap::new(),

            settings_map: HashMap::new(),
            raw_settings_map: HashMap::new(),
            variables_map: HashMap::new(),
            repo_map: HashMap::new(),
        }
    }
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct RawArtifact {
    pub partof: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, RustcEncodable, RustcDecodable)]
pub struct RawSettings {
    pub disabled: Option<bool>,
    pub artifact_paths: Option<Vec<String>>,
    pub code_paths: Option<Vec<String>>,
    pub exclude_code_paths: Option<Vec<String>>,
    pub color: Option<bool>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
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
// note: implementation of methods in name.rs
#[derive(Clone)]
pub struct ArtName {
    pub raw: String,
    pub value: Vec<String>,
    pub ty: ArtType,
}

impl Text {
    pub fn new(raw: &str) -> Text {
        Text {
            raw: raw.to_string(),
            value: raw.to_string(),
        }
    }
}

/// The Artifact type. This encapsulates
/// REQ, SPC, RSK, and TST artifacts and
/// contains space to link them
#[derive(Clone, Debug)]
pub struct Artifact {
    // directly loaded types
    pub path: PathBuf,
    pub text: Text,
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

    pub fn to_data(&self, name: &ArtNameRc) -> ArtifactData {
        ArtifactData {
            id: get_unique_id() as u64,
            name: name.raw.clone(),
            path: self.path.to_string_lossy().to_string(),
            text: self.text.clone(),
            partof: self.partof.iter().map(|n| n.raw.clone()).collect(),
            parts: self.parts.iter().map(|n| n.raw.clone()).collect(),
            loc: self.loc.as_ref().map(
                |l| LocData {path: l.path.to_string_lossy().to_string(), 
                             row: l.line_col.0 as u64, col: l.line_col.1 as u64}),
            completed: self.completed,
            tested: self.tested,
        }
    }

    pub fn from_data(data: &ArtifactData) -> Result<(ArtNameRc, Artifact)> {
        let name = try!(ArtNameRc::from_str(&data.name));
        let mut partof: HashSet<ArtNameRc> = HashSet::new();
        for p in &data.partof {
            let pname = try!(ArtNameRc::from_str(p));
            partof.insert(pname);
        }
        Ok((name, Artifact {
            path: PathBuf::from(&data.path),
            text: data.text.clone(),
            partof: partof,
            loc: None,
            parts: HashSet::new(),
            completed: -1.0,
            tested: -1.0,
        }))
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

#[cfg(not(windows))]
const DEFAULT_COLOR: bool = true;

#[cfg(windows)]
const DEFAULT_COLOR: bool = false;

impl Settings {
    pub fn new() -> Settings {
        Settings {
            disabled: false,
            paths: VecDeque::new(),
            code_paths: VecDeque::new(),
            exclude_code_paths: VecDeque::new(),
            color: DEFAULT_COLOR,
        }
    }
}

