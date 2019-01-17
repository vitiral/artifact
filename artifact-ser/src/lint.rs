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
use std::error;
use std::fmt;
/// #SPC-lint
/// Artifact lint types.
///
/// This is the primary error type for all "non fatal" errors and warnings.
use std::sync::mpsc::Sender;

use dev_prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// An artifact lint error or warning
pub struct Lint {
    pub level: Level,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<String>,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub line: Option<u64>,
    pub category: Category,
    pub msg: String,
}

#[derive(Debug, Default, PartialEq, Serialize, Deserialize)]
/// Categorized lints
pub struct Categorized {
    pub error: Vec<Lint>,
    pub other: Vec<Lint>,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// The lint level, error will eventually stop the program.
pub enum Level {
    Error,
    Warn,
}

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// Where the lint is from
pub enum Category {
    LoadPaths,
    ParseCodeImplementations,
    ParseArtifactFiles,
    AutoPartof,
    Artifact,
    Settings,
    ImplCode,
    ModifyPathInvalid,
    CreateExists,
    UpdateDne,
    DeleteDne,
    IdOverlap,
    CreateBackups,
    SaveProject,
    RemoveBackups,
}

impl Categorized {
    pub fn categorize<I>(&mut self, lints: I)
    where
        I: Iterator<Item = Lint>,
    {
        for lint in lints {
            match lint.level {
                Level::Error => self.error.push(lint),
                _ => self.other.push(lint),
            }
        }
    }

    /// sort and dedup the internal lists
    pub fn sort(&mut self) {
        self.error.sort();
        self.error.dedup();

        self.other.sort();
        self.other.dedup();
    }

    /// Return whether there are _any_ lints.
    pub fn is_empty(&self) -> bool {
        self.error.is_empty() && self.other.is_empty()
    }
}

impl error::Error for Categorized {
    fn description(&self) -> &str {
        "multiple lint errors, both errors and warnings"
    }
}

impl fmt::Display for Categorized {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(
            f,
            "  ----- ERRORS -----:\n{}\n\n",
            expect!(json::to_string_pretty(&self.error))
        )?;
        write!(
            f,
            "  ----- WARNINGS -----:\n{}\n",
            expect!(json::to_string_pretty(&self.other))
        )
    }
}

impl Lint {
    pub fn load_error<S: ToString>(path: S, err: &str) -> Lint {
        Lint {
            level: Level::Error,
            category: Category::LoadPaths,
            path: Some(path.to_string()),
            line: None,
            msg: format!("Error during loading: {}", err),
        }
    }

    pub fn create_exists(err: String) -> Lint {
        Lint {
            level: Level::Error,
            category: Category::CreateExists,
            path: None,
            line: None,
            msg: err,
        }
    }

    pub fn update_dne(err: String) -> Lint {
        Lint {
            level: Level::Error,
            category: Category::UpdateDne,
            path: None,
            line: None,
            msg: err,
        }
    }

    pub fn delete_dne(err: String) -> Lint {
        Lint {
            level: Level::Error,
            category: Category::DeleteDne,
            path: None,
            line: None,
            msg: err,
        }
    }

    pub fn id_overlap(err: String) -> Lint {
        Lint {
            level: Level::Error,
            category: Category::IdOverlap,
            path: None,
            line: None,
            msg: err,
        }
    }
}

pub fn io_error<S: ToString>(lints: &Sender<Lint>, path: S, err: &str) {
    lints
        .send(Lint::load_error(path, err))
        .expect("failed to send io-error");
}
