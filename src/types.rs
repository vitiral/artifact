use dev_prefix::*;

pub const VERSION: &'static str = env!("CARGO_PKG_VERSION");
/// variable which can be used in settings path to mean the repo directory
pub const REPO_VAR: &'static str = "repo";
/// variable which can be used in settings paths to mean the dir of the settings file.
/// #TODO: remove this
pub const CWD_VAR: &'static str = "cwd";
/// base definition of a valid name. Some pieces may ignore case.
pub const NAME_VALID_STR: &'static str = "(?:REQ|SPC|RSK|TST)(?:-[A-Z0-9_-]*[A-Z0-9_])?";

lazy_static!{
    // must start with artifact type, followed by "-", followed by at least 1 valid character
    // cannot end with "-"
    pub static ref NAME_VALID: Regex = Regex::new(
        &format!("^{}$", NAME_VALID_STR)).unwrap();
    pub static ref PARENT_PATH: PathBuf = PathBuf::from("PARENT");
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
        TomlDecode(::toml::DecodeError);
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

        // Processing errors
        InvalidTextVariables {
            description("couldn't resolve some text variables")
        }
        InvalidPartof {
            description("Some artifacts have invalid partof attributes")
        }
        InvalidDone {
            description("Some artifacts have invalid partof attributes")
        }
        NameNotFound(desc: String) {
            description("searched for names were not found")
            display("the following artifacts do not exists: {}", desc)
        }
        LocNotFound {
            description("errors while finding implementation locations")
        }
        DoneTwice(desc: String) {
            description("the artifact is done and implemented in code")
            display("referenced in code and `done` is set: {}", desc)
        }
        InvalidUnicode(path: String) {
            description("we do not yet support non-unicode paths")
            display("invalid unicode in path: {}", path)
        }

        // Cmd errors
        CmdError(desc: String) {
            description("error while running a command")
            display("{}", desc)
        }

        // Misc errors
        PathNotFound(desc: String) {
            description("invalid path")
            display("Path does not exist: {}", desc)
        }
        NotEqual(desc: String) {
            description("values not equal")
            display("{}", desc)
        }
        Security(desc: String) {
            description("security vulnerability detected")
            display("security vulnerability: {}", desc)
        }
        Internal(desc: String) {
            description("internal error")
            display("internal error: {}", desc)
        }
        NothingDone {
            description("internal control flow")
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

            origin: PARENT_PATH.to_path_buf(),
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
    RSK,
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

impl Done {
    /// return true if Done == Code || Defined
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
    pub partof: Names,
    pub parts: Names,
    pub done: Done,
    pub completed: f32, // completed ratio (calculated)
    pub tested: f32, // tested ratio (calculated)
}

impl Artifact {
    /// artifact was automatically created as a parent
    pub fn is_parent(&self) -> bool {
        self.path == PARENT_PATH.as_path()
    }
}

/// repo settings for loading artifacts
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

