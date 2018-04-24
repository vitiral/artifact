//! These are purely copies of their more "full" types with the paths, etc removed.

use fmt;

use dev_prelude::*;
use name::{Name, SubName};
use lint;
use super::{Completed, HashIm};

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectResultSer {
    pub project: ProjectSer,
    pub lints: lint::Categorized,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum ArtifactOpSer {
    Create {
        artifact: ArtifactImSer,
    },
    Update {
        artifact: ArtifactImSer,
        name: Name,
        id: HashIm,
    },
    Delete {
        name: Name,
        id: HashIm,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectSer {
    pub paths: ProjectPathsSer,
    pub code_impls: IndexMap<Name, ImplCodeSer>,
    pub artifacts: IndexMap<Name, ArtifactSer>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct ProjectPathsSer {
    pub base: String,
    pub code_paths: IndexSet<String>,
    pub exclude_code_paths: IndexSet<String>,
    pub artifact_paths: IndexSet<String>,
    pub exclude_artifact_paths: IndexSet<String>,
}

#[derive(Debug, PartialEq, Clone, Serialize, Deserialize)]
pub struct ArtifactSer {
    pub id: HashIm,
    pub name: Name,
    pub file: String,
    pub partof: IndexSet<Name>,
    pub parts: IndexSet<Name>,
    pub completed: Completed,
    pub text: String,
    pub impl_: ImplSer,
    pub subnames: IndexSet<SubName>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(tag = "type", content = "value")]
pub enum ImplSer {
    Done(String),
    Code(ImplCodeSer),
    NotImpl,
}

impl ImplSer {
    /// Return whether this is the `Done` variant.
    pub fn is_done(&self) -> bool {
        match *self {
            ImplSer::Done(_) => true,
            _ => false,
        }
    }

    /// Represent the impl_ as either `done` or nothing.
    ///
    /// Used to get the default of "done".
    pub fn as_done(&self) -> Option<&str> {
        if let ImplSer::Done(ref d) = *self {
            Some(d)
        } else {
            None
        }
    }
}

impl fmt::Display for ImplSer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match *self {
            ImplSer::Done(ref s) => write!(f, "{}", s),
            ImplSer::Code(ref c) => write!(f, "{}", c),
            ImplSer::NotImpl => write!(f, "not directly implemented"),
        }
    }
}

impl fmt::Display for ImplCodeSer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        if let Some(ref loc) = self.primary {
            write!(f, "{:?}", loc)?;
        }
        if !self.secondary.is_empty() {
            write!(f, "Secondary{:?}", self.secondary)?;
        }
        Ok(())
    }
}

impl fmt::Debug for CodeLocSer {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}[{}]", self.file, self.line)
    }
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ImplCodeSer {
    pub primary: Option<CodeLocSer>,
    pub secondary: IndexMap<SubName, CodeLocSer>,
}

#[derive(Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct CodeLocSer {
    pub file: String,
    pub line: u64,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactImSer {
    pub name: Name,
    pub file: String,
    pub partof: IndexSet<Name>,
    pub done: Option<String>,
    pub text: String,
}

