/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2017  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */
//! project wide types

use dev_prefix::*;
use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};

pub const REPO_VAR: &'static str = "repo";
pub const CWD_VAR: &'static str = "cwd";
pub const ART_VALID_STR: &'static str = "(?:REQ|SPC|RSK|TST)(?:-[A-Z0-9_-]*[A-Z0-9_])?";

lazy_static!{
    // must start with artifact type, followed by "-", followed by at least 1 valid character
    // cannot end with "-"
    pub static ref ART_VALID: Regex = Regex::new(
        &format!("^{}$", ART_VALID_STR)).unwrap();
    pub static ref PARENT_PATH: PathBuf = PathBuf::from("PARENT");
    pub static ref INCREMENTING_ID: AtomicUsize = AtomicUsize::new(0);
}

pub type Artifacts = HashMap<ArtNameRc, Artifact>;
pub type ArtNameRc = Arc<ArtName>;
pub type ArtNames = HashSet<ArtNameRc>;

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
    pub files: HashSet<PathBuf>,
    pub dne_locs: HashMap<ArtName, Loc>,

    // preserved locations where each piece is from
    pub origin: PathBuf,
    pub repo_map: HashMap<PathBuf, PathBuf>,
}

impl Default for Project {
    fn default() -> Project {
        Project {
            artifacts: Artifacts::default(),
            settings: Settings::default(),
            files: HashSet::default(),
            dne_locs: HashMap::default(),

            origin: PARENT_PATH.to_path_buf(),
            repo_map: HashMap::default(),
        }
    }
}

fn names_equal(a: &Artifacts, b: &Artifacts) -> Result<()> {
    let a_keys: HashSet<ArtNameRc> = a.keys().cloned().collect();
    let b_keys: HashSet<ArtNameRc> = b.keys().cloned().collect();
    if b_keys != a_keys {
        let missing = a_keys.symmetric_difference(&b_keys)
            .collect::<Vec<_>>();
        let msg = format!("missing artifacts: {:?}\nFIRST:\n{:?}\nSECOND:\n{:?}",
                          missing,
                          a_keys,
                          b_keys);
        Err(ErrorKind::NotEqual(msg).into())
    } else {
        Ok(())
    }
}

/// assert that the attributes are equal on the artifact
/// if they are not, then find what is different and include
/// in the error description.
///
/// This is very expensive for values that differ
fn attr_equal<T, F>(attr: &str, a: &Artifacts, b: &Artifacts, get_attr: &F) -> Result<()>
    where T: Debug + PartialEq,
          F: Fn(&Artifact) -> &T
{
    let mut diff: Vec<String> = Vec::new();

    for (a_name, a_art) in a.iter() {
        let b_art = b.get(a_name).unwrap();
        let a_attr = get_attr(a_art);
        let b_attr = get_attr(b_art);
        if a_attr != b_attr {
            let mut a_str = format!("{:?}", a_attr);
            let mut b_str = format!("{:?}", b_attr);
            let a_big = if a_str.len() > 100 { "..." } else { "" };
            let b_big = if b_str.len() > 100 { "..." } else { "" };
            a_str.truncate(100);
            b_str.truncate(100);
            diff.push(format!("[{}: {}{} != {}{}]", a_name, a_str, a_big, b_str, b_big));
        }
    }

    if diff.is_empty() {
        Ok(())
    } else {
        Err(ErrorKind::NotEqual(format!("{} different: {:?}", attr, diff)).into())
    }
}

/// num *approximately* equal
fn float_equal<F>(attr: &str, a: &Artifacts, b: &Artifacts, get_num: &F) -> Result<()>
    where F: Fn(&Artifact) -> f32
{
    let mut diff: Vec<String> = Vec::new();
    fn thous(f: f32) -> i64 {
        (f * 1000.) as i64
    }

    for (a_name, a_art) in a.iter() {
        let b_art = b.get(a_name).unwrap();
        let a_attr = get_num(a_art);
        let b_attr = get_num(b_art);
        if thous(a_attr) != thous(b_attr) {
            let mut a_str = format!("{:?}", a_attr);
            let mut b_str = format!("{:?}", b_attr);
            a_str.truncate(50);
            b_str.truncate(50);
            diff.push(format!("({}, {} != {})", a_name, a_str, b_str));
        }
    }

    if diff.is_empty() {
        Ok(())
    } else {
        Err(ErrorKind::NotEqual(format!("{} different: {:?}", attr, diff)).into())
    }
}

fn proj_attr_equal<T>(attr: &str, a: &T, b: &T) -> Result<()>
    where T: Debug + PartialEq
{
    if a != b {
        Err(ErrorKind::NotEqual(format!("{} FIRST:\n{:?}\n\nSECOND:\n{:?}", attr, a, b)).into())
    } else {
        Ok(())
    }
}


impl Project {
    /// better than equal... has reasons why NOT equal!
    pub fn equal(&self, other: &Project) -> Result<()> {
        names_equal(&self.artifacts, &other.artifacts)?;
        attr_equal("path",
                   &self.artifacts,
                   &other.artifacts,
                   &|a: &Artifact| &a.path)?;
        attr_equal("text",
                   &self.artifacts,
                   &other.artifacts,
                   &|a: &Artifact| &a.text)?;
        attr_equal("partof",
                   &self.artifacts,
                   &other.artifacts,
                   &|a: &Artifact| &a.partof)?;
        attr_equal("parts",
                   &self.artifacts,
                   &other.artifacts,
                   &|a: &Artifact| &a.parts)?;
        attr_equal("done",
                   &self.artifacts,
                   &other.artifacts,
                   &|a: &Artifact| &a.done)?;
        float_equal("completed",
                    &self.artifacts,
                    &other.artifacts,
                    &|a: &Artifact| a.completed)?;
        float_equal("tested",
                    &self.artifacts,
                    &other.artifacts,
                    &|a: &Artifact| a.tested)?;
        proj_attr_equal("origin", &self.origin, &other.origin)?;
        proj_attr_equal("settings", &self.settings, &other.settings)?;
        proj_attr_equal("files", &self.files, &other.files)?;
        proj_attr_equal("dne_locs", &self.dne_locs, &other.dne_locs)?;
        proj_attr_equal("repo_map", &self.repo_map, &other.repo_map)?;
        Ok(())
    }
}

/// struct for representing a project as just a collection of
/// Path and String values, used for loading/formatting/saving files
#[derive(Debug, PartialEq)]
pub struct ProjectText {
    pub origin: PathBuf,
    pub files: HashMap<PathBuf, String>,
}

impl Default for ProjectText {
    fn default() -> ProjectText {
        ProjectText {
            origin: PARENT_PATH.to_path_buf(),
            files: HashMap::default(),
        }
    }
}

#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct RawArtifact {
    pub partof: Option<String>,
    pub text: Option<String>,
    pub done: Option<String>,
}

#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct RawSettings {
    pub artifact_paths: Option<Vec<String>>,
    pub code_paths: Option<Vec<String>>,
    pub exclude_code_paths: Option<Vec<String>>,
    pub additional_repos: Option<Vec<String>>,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
pub enum ArtType {
    REQ,
    SPC,
    RSK,
    TST,
}


/// Definition of an artifact name, with Traits for hashing,
/// displaying, etc
// note: methods are implemented in name.rs
#[derive(Clone)]
pub struct ArtName {
    pub raw: String,
    pub value: Vec<String>,
    pub ty: ArtType,
}

#[derive(Debug, Clone, PartialEq)]
pub struct Loc {
    pub path: PathBuf,
    pub line: usize,
}

impl Loc {
    pub fn fake() -> Loc {
        Loc {
            path: Path::new("fake").to_path_buf(),
            line: 42,
        }
    }
}

impl fmt::Display for Loc {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}[{}]", self.path.display(), self.line)
    }
}


/// is the artifact "done by definition"
/// It is done by definition if:
/// - it is found in source code
/// - it has it's `done` field set
#[derive(Debug, Clone, PartialEq)]
pub enum Done {
    /// Artifact is implemented in code
    Code(Loc),
    /// artifact has it's `done` field defined
    Defined(String),
    /// artifact is NOT "done by definition"
    NotDone,
}

impl Done {
    pub fn is_done(&self) -> bool {
        match *self {
            Done::Code(_) | Done::Defined(_) => true,
            Done::NotDone => false,
        }
    }
}

impl fmt::Display for Done {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            Done::Code(ref c) => write!(f, "{}", c),
            Done::Defined(ref s) => write!(f, "{}", s),
            Done::NotDone => write!(f, "not done"),
        }
    }
}

/// The Artifact type. This encapsulates
/// REQ, SPC, RSK, and TST artifacts and
/// contains space to link them
/// #SPC-artifact
#[derive(Clone, Debug, PartialEq)]
pub struct Artifact {
    // directly loaded types
    pub path: PathBuf,
    pub text: String,
    pub partof: ArtNames,
    pub parts: ArtNames,
    pub done: Done,
    pub completed: f32, // completed ratio (calculated)
    pub tested: f32, // tested ratio (calculated)
}

impl Artifact {
    pub fn is_parent(&self) -> bool {
        self.path == PARENT_PATH.as_path()
    }

    pub fn to_data(&self, name: &ArtNameRc) -> ArtifactData {
        let (code, done) = match self.done {
            Done::Code(ref l) => {
                (Some(LocData {
                     path: l.path.to_string_lossy().to_string(),
                     line: l.line as u64,
                 }),
                 None)
            }
            Done::Defined(ref s) => (None, Some(s.clone())),
            Done::NotDone => (None, None),
        };
        ArtifactData {
            id: get_unique_id() as u64,
            name: name.raw.clone(),
            path: self.path.to_string_lossy().to_string(),
            text: self.text.clone(),
            partof: self.partof.iter().map(|n| n.raw.clone()).collect(),
            parts: self.parts.iter().map(|n| n.raw.clone()).collect(),
            code: code,
            done: done,
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
        let done = if data.done.is_some() && data.code.is_some() {
            let msg = "has both done and code defined".to_string();
            return Err(ErrorKind::InvalidArtifact(data.name.clone(), msg).into());
        } else if let Some(ref d) = data.done {
            Done::Defined(d.clone())
        } else if let Some(ref c) = data.code {
            Done::Code(Loc {
                path: PathBuf::from(&c.path),
                line: c.line as usize,
            })
        } else {
            Done::NotDone
        };
        Ok((name,
            Artifact {
                path: PathBuf::from(&data.path),
                text: data.text.clone(),
                partof: partof,
                done: done,
                parts: HashSet::new(),
                completed: -1.0,
                tested: -1.0,
            }))
    }
}


#[derive(Debug, Default, Clone, PartialEq)]
pub struct Settings {
    pub artifact_paths: HashSet<PathBuf>,
    pub code_paths: VecDeque<PathBuf>,
    pub exclude_code_paths: VecDeque<PathBuf>,
    pub additional_repos: VecDeque<PathBuf>,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            artifact_paths: HashSet::new(),
            code_paths: VecDeque::new(),
            exclude_code_paths: VecDeque::new(),
            additional_repos: VecDeque::new(),
        }
    }
}

// ##################################################
// Serialized Data Types

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct LocData {
    pub path: String,
    pub line: u64,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ArtifactData {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub text: String,
    pub partof: Vec<String>,

    // // TODO: until I serde gets up to speed, the web-api will
    // // have to send these values even though they are ignored
    //#[serde(default)]
    pub parts: Vec<String>,
    //#[serde(default)]
    pub code: Option<LocData>,
    //#[serde(default)]
    pub done: Option<String>,
    //#[serde(default = -1)]
    pub completed: f32,
    //#[serde(default = -1)]
    pub tested: f32,
}

#[allow(non_camel_case_types)]
pub enum RpcErrors {
    xIdsNotFound,
    xFilesNotFound,
    xInvalidName,
    xInvalidPartof,
}
