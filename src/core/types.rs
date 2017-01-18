/*  rst: the requirements tracking tool made for developers
 * Copyright (C) 2016  Garrett Berg <@vitiral, vitiral@gmail.com>
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
pub use super::{ArtifactData, LocData, Text};

pub type Artifacts = HashMap<ArtNameRc, Artifact>;
pub type ArtNameRc = Arc<ArtName>;
pub type ArtNames = HashSet<ArtNameRc>;
pub type Variables = HashMap<String, String>;


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
#[derive(Debug, Default, Clone)]
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

fn names_equal(a: &Artifacts, b: &Artifacts) -> Result<()> {
    let a_keys: HashSet<ArtNameRc> = a.keys().cloned().collect();
    let b_keys: HashSet<ArtNameRc> = b.keys().cloned().collect();
    if b_keys != a_keys {
        let missing = a_keys.symmetric_difference(&b_keys)
            .collect::<Vec<_>>();
        Err(ErrorKind::NotEqual(format!("missing names: {:?}", missing)).into())
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
            a_str.truncate(50);
            b_str.truncate(50);
            diff.push(format!("({}, {}!={})", a_name, a_str, b_str));
        }
    }

    if diff.is_empty() {
        Ok(())
    } else {
        Err(ErrorKind::NotEqual(format!("{} different: {:?}", attr, diff)).into())
    }
}

/// num *approximately* equal
fn num_equal<F>(attr: &str, a: &Artifacts, b: &Artifacts, get_num: &F) -> Result<()>
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
            diff.push(format!("({}, {}!={})", a_name, a_str, b_str));
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
        attr_equal("loc",
                   &self.artifacts,
                   &other.artifacts,
                   &|a: &Artifact| &a.loc)?;
        num_equal("completed",
                  &self.artifacts,
                  &other.artifacts,
                  &|a: &Artifact| a.completed)?;
        num_equal("tested",
                  &self.artifacts,
                  &other.artifacts,
                  &|a: &Artifact| a.tested)?;
        proj_attr_equal("settings", &self.settings, &other.settings)?;
        proj_attr_equal("variables", &self.variables, &other.variables)?;
        proj_attr_equal("files", &self.files, &other.files)?;
        proj_attr_equal("dne_locs", &self.dne_locs, &other.dne_locs)?;
        proj_attr_equal("settings_map", &self.settings_map, &other.settings_map)?;
        proj_attr_equal("raw_settings_map",
                        &self.raw_settings_map,
                        &other.raw_settings_map)?;
        proj_attr_equal("variables_map", &self.variables_map, &other.variables_map)?;
        proj_attr_equal("repo_map", &self.repo_map, &other.repo_map)?;
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct RawArtifact {
    pub partof: Option<String>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
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
        write!(f,
               "{}({}:{})",
               self.path.display(),
               self.line_col.0,
               self.line_col.1)
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
#[derive(Clone, Debug, PartialEq)]
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
            loc: self.loc.as_ref().map(|l| {
                LocData {
                    path: l.path.to_string_lossy().to_string(),
                    row: l.line_col.0 as u64,
                    col: l.line_col.1 as u64,
                }
            }),
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
        Ok((name,
            Artifact {
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


#[derive(Debug, Default, Clone, PartialEq)]
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
