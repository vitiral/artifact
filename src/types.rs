#![allow(unused_doc_comment)]

use dev_prefix::*;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
/// variable which can be used in settings path to mean the repo directory
pub const REPO_VAR: &'static str = "repo";
/// variable which can be used in settings paths to mean the dir of the settings file.
/// #TODO: remove this
pub const CWD_VAR: &'static str = "cwd";
/// base definition of a valid name. Some pieces may ignore case.
pub const NAME_VALID_STR: &'static str = "(?:REQ|SPC|TST)(?:-[A-Z0-9_-]*[A-Z0-9_])?";
pub const SUBPART_VALID_STR: &'static str = "(?:[A-Z0-9_-]*[A-Z0-9_])?";

lazy_static!{
    // must start with artifact type, followed by "-", followed by at least 1 valid character
    // cannot end with "-"
    pub static ref NAME_VALID: Regex = Regex::new(
        &format!("^{}$", NAME_VALID_STR)).unwrap();
    pub static ref REPO_DIR: PathBuf = PathBuf::from(".art");
    pub static ref SETTINGS_PATH: PathBuf = REPO_DIR.join("settings.toml");
}

error_chain! {
    types {
        Error, ErrorKind, ResultExt, Result;
    }

    links {
        // no external error chains (yet)
    }

    foreign_links {
        // stdlib
        Io(::std::io::Error);
        Fmt(::std::fmt::Error);

        // crates
        StrFmt(::strfmt::FmtError);
        TomlError(::toml::de::Error);
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
            description("Invalid artifact name")
            display("Invalid artifact name: \"{}\"", desc)
        }
        InvalidAttr(name: String, attr: String) {
            description("Artifact has invalid attribute")
            display("Artifact {} has invalid attribute: {}", name, attr)
        }
        InvalidSettings(desc: String) {
            description("Invalid settings")
            display("Invalid settings: {}", desc)
        }
        InvalidArtifact(name: String, desc: String) {
            description("Invalid artifact")
            display("Artifact {} is invalid: {}", name, desc)
        }
        MissingParent(name: String, parent: String) {
            description("Missing parent artifact")
            display("Parent {} does not exist for {}", parent, name)
        }
        // Processing errors
        InvalidTextVariables {
            description("Couldn't resolve some text variables")
        }
        InvalidPartof {
            description("Some artifacts have invalid partof attributes")
        }
        InvalidDone {
            description("Some artifacts have invalid partof attributes")
        }
        NameNotFound(desc: String) {
            description("Searched for names were not found")
            display("The following artifacts do not exists: {}", desc)
        }
        LocNotFound {
            description("Errors while finding implementation locations")
        }
        DoneTwice(desc: String) {
            description("The artifact is done and implemented in code")
            display("Referenced in code and `done` is set: {}", desc)
        }
        InvalidUnicode(path: String) {
            description("We do not yet support non-unicode paths")
            display("Invalid unicode in path: {}", path)
        }

        // Cmd errors
        CmdError(desc: String) {
            description("Error while running a command")
            display("{}", desc)
        }

        // Misc errors
        PathNotFound(desc: String) {
            description("Invalid path")
            display("Path does not exist: {}", desc)
        }
        NotEqual(desc: String) {
            description("Values not equal")
            display("{}", desc)
        }
        Security(desc: String) {
            description("Security vulnerability detected")
            display("Security vulnerability: {}", desc)
        }
        Internal(desc: String) {
            description("Internal error")
            display("Internal error: {}", desc)
        }
        NothingDone {
            description("Internal control flow")
        }
    }
}

/// our `from_str` can throw errors
pub trait LoadFromStr: Sized {
    fn from_str(s: &str) -> Result<Self>;
}

/// Artifacts organized by name
pub type Artifacts = HashMap<NameRc, Artifact>;
/// Names in a `HashSet` for fast lookup
pub type Names = HashSet<NameRc>;
pub type NameRc = Arc<Name>;

/// represents the results and all the data necessary
/// to reconstruct a loaded project
#[derive(Debug, Clone)]
pub struct Project {
    pub artifacts: Artifacts,
    pub settings: Settings,
    pub files: HashSet<PathBuf>,
    pub dne_locs: HashMap<Name, Loc>,

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
            origin: PathBuf::default(),
            repo_map: HashMap::default(),
        }
    }
}

/// Definition of an artifact name, with Traits for hashing,
/// displaying, etc
// note: methods are implemented in name.rs
#[derive(Clone)]
pub struct Name {
    /// user definition
    pub raw: String,
    /// standardized version
    pub value: Vec<String>,
    /// the inferred type of the artifact
    pub ty: Type,
}

#[derive(Debug, Copy, Clone, Eq, PartialEq, Hash)]
/// type of an `Artifact`
pub enum Type {
    REQ,
    SPC,
    TST,
}

/// location in a file
#[derive(Debug, Clone, PartialEq)]
pub struct Loc {
    pub path: PathBuf,
    pub line: usize,
}

#[cfg(test)]
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

/// multiple locations
#[derive(Debug, Clone, PartialEq)]
pub struct Locs {
    /// Whether the root node is linked in code
    /// i.e #ART-foo
    pub root: Option<loc>,

    /// The sub parts that are linked in code
    /// i.e #ART-foo.subpart
    pub subparts: HashMap<String, Option<Loc>>,
}

impl Locs {
    /// Give the ratio of these locations are complete
    pub fn ratio_complete(&self) -> f32 {
        let total = 1 + self.subparts.len();
        let mut linked: usize = self.subparts.iter()
            .map(|(k, v)| if v.is_some() { 1 } else { 0 })
            .sum();
        linked += if self.root.is_some() { 1 } else { 0 };

        linked as f32 / total
    }
}

#[cfg(test)]
impl Locs {
    pub fn fake() -> Loc {
        Locs {
            root: Loc::fake(),
            subparts: Vec::new(),
            num_subparts: 0,
        }
    }
}

impl fmt::Display for Locs {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        // TODO: need better handling of this
        match root {
            Some(r) => write!(f, "{}", self.root),
            None => write!(f, "! only subparts are linked !"),
        }
    }
}


/// Determines if the artifact is "done by definition"
///
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
/// REQ, SPC, and TST artifacts and
/// contains space to link them
/// #SPC-artifact
#[derive(Clone, Debug, PartialEq)]
pub struct Artifact {
    /// constant id for this instance
    pub id: u64,
    /// revision id for edit functionality
    pub revision: u64,
    /// path of definition (.toml file)
    pub def: PathBuf,
    /// `text` attr
    pub text: String,
    /// explicit and calculated `partof` attribute
    pub partof: Names,
    /// parts is inverse of partof (calculated)
    pub parts: Names,
    /// sublinks that are defined only in code (for reference)
    pub sublinks: HashSet<String>,
    /// `done` attribute, allows user to "define as done"
    pub done: Done,
    /// completed ratio (calculated)
    pub completed: f32,
    /// tested ratio (calculated)
    pub tested: f32,
}

/// repo settings for loading artifacts
/// #SPC-project-settings
#[derive(Debug, Default, Clone, PartialEq)]
pub struct Settings {
    pub artifact_paths: HashSet<PathBuf>,
    pub exclude_artifact_paths: HashSet<PathBuf>,
    pub code_paths: HashSet<PathBuf>,
    pub exclude_code_paths: HashSet<PathBuf>,
}

impl Settings {
    pub fn new() -> Settings {
        Settings {
            artifact_paths: HashSet::new(),
            exclude_artifact_paths: HashSet::new(),
            code_paths: HashSet::new(),
            exclude_code_paths: HashSet::new(),
        }
    }
}

#[derive(Debug, Default, Clone, Serialize)]
/// struct that is passed to the api server
pub struct ServeCmd {
    pub addr: String,
    pub readonly: bool,
    pub path_url: String,
}
