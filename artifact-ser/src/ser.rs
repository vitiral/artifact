/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018 Rett Berg <@vitiral, vitiral@gmail.com>
 *
 * The source code is Licensed under either of
 *
 * * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
 *   http://www.apache.org/licenses/LICENSE-2.0)
 * * MIT license ([LICENSE-MIT](LICENSE-MIT) or
 *   http://opensource.org/licenses/MIT)
 *
 * at your option.
 *
 * Unless you explicitly state otherwise, any contribution intentionally submitted
 * for inclusion in the work by you, as defined in the Apache-2.0 license, shall
 * be dual licensed as above, without any additional terms or conditions.
 * */
//! These are purely copies of their more "full" types with the paths, etc removed.

use fmt;

use dev_prelude::*;
use name::{Name, SubName};
use lint;
use super::{Completed, HashIm, SettingsExport};

/// The initial state of the project, stored in an `initial.json` file.
#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectInitialSer {
    pub project: Option<ProjectSer>,
    pub web_type: WebType,
}

/// The type of Web Edit that the project is.
#[derive(Debug, Serialize, Deserialize, Eq, PartialEq)]
pub enum WebType {
    /// The project is editable and can be reloaded.
    Editable,
    /// The project is readonly but can be reloaded.
    Readonly,
    /// The project is completely static, i.e. exported.
    Static,
}

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
        orig_id: HashIm,
    },
    Delete {
        name: Name,
        orig_id: HashIm,
    },
}

#[derive(Debug, PartialEq, Serialize, Deserialize)]
pub struct ProjectSer {
    pub settings: SettingsSer,
    pub code_impls: IndexMap<Name, ImplCodeSer>,
    pub artifacts: IndexMap<Name, ArtifactSer>,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
pub struct SettingsSer {
    pub base: String,
    pub code_paths: IndexSet<String>,
    pub exclude_code_paths: IndexSet<String>,
    pub artifact_paths: IndexSet<String>,
    pub exclude_artifact_paths: IndexSet<String>,

    // command specific settings
    #[serde(default)]
    pub export: SettingsExport,
}


impl ProjectSer {
    /// Get the name/subname code location if they exist.
    pub fn get_impl(&self, name: &str, sub: Option<&str>) -> Result<&CodeLocSer, String> {
        let name = Name::from_str(name).map_err(|e| e.to_string())?;
        let code = self.code_impls.get(&name)
            .ok_or_else(|| format!("{} not implemented", name))?;
        match sub {
            None => {
                match code.primary {
                    Some(ref c) => Ok(c),
                    None => Err("not implemented".into()),
                }
            }
            Some(sub) => {
                let sub = SubName::from_str(sub).map_err(|e| e.to_string())?;
                code.secondary.get(&sub)
                    .ok_or_else(|| format!("{}.{} not implemented", name, sub))
            },
        }
    }
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

