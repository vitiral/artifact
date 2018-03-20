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
/// #SPC-lint
/// Artifact lint types.
///
/// This is the primary error type for all "non fatal" errors and warnings.
use std::sync::mpsc::Sender;
use std::error;
use std::fmt;

use dev_prelude::*;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// An artifact lint error or warning
pub struct Lint {
    pub level: Level,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub path: Option<PathArc>,
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
    ImplCode,
    ModifyPathInvalid,
    CreateExists,
    UpdateNoop,
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
    pub fn load_error<P: AsRef<Path>>(path: P, err: &str) -> Lint {
        Lint {
            level: Level::Error,
            category: Category::LoadPaths,
            path: Some(PathArc::new(path)),
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

    pub fn update_noop(err: String) -> Lint {
        Lint {
            level: Level::Error,
            category: Category::UpdateNoop,
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

pub fn io_error<P: AsRef<Path>>(lints: &Sender<Lint>, path: P, err: &str) {
    lints
        .send(Lint::load_error(path, err))
        .expect("failed to send io-error");
}
