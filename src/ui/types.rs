use dev_prefix::*;
use types::*;

/// settings for what to format
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct FmtSettings {
    pub long: bool,
    pub recurse: u8,
    pub def: bool,
    pub parts: bool,
    pub partof: bool,
    pub loc_path: bool,
    pub text: bool,
    pub color: bool,
}

impl FmtSettings {
    pub fn is_empty(&self) -> bool {
        !self.long && !self.def && !self.parts && !self.partof && !self.loc_path && !self.text
    }
}

/// structure which contains all the information necessary to
/// format an artifact for cmdline, html, or anything else
/// purposely doesn't contain items that are *always* displayed
/// such as completed or tested
#[derive(Debug, Default)]
pub struct FmtArtifact {
    pub long: bool,
    pub def: Option<PathBuf>,
    pub parts: Option<Vec<FmtArtifact>>,
    pub partof: Option<Vec<FmtArtifact>>,
    pub done: Option<String>,
    // pub loc_path: Option<PathBuf>,
    // pub loc_line_col: (usize, usize),
    // pub loc_valid: Option<bool>,
    pub text: Option<String>,
    pub name: NameRc,
}


#[derive(Debug, PartialEq, Eq)]
pub struct PercentSearch {
    pub lt: bool,
    pub perc: i8,
}

impl Default for PercentSearch {
    fn default() -> PercentSearch {
        PercentSearch {
            lt: false,
            perc: -127,
        }
    }
}


#[derive(Debug, Default, PartialEq, Eq)]
pub struct SearchSettings {
    pub use_regex: bool,
    pub name: bool,
    pub def: bool,
    pub parts: bool,
    pub partof: bool,
    pub loc: bool,
    pub text: bool,
    pub completed: PercentSearch,
    pub tested: PercentSearch,
}
