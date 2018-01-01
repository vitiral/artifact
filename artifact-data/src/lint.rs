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
/// Artifact lint types.
///
/// This is the primary error type for all "non fatal" errors and warnings.
use std::sync::mpsc::Sender;

use dev_prelude::*;
use path_abs::PathAbs;

#[derive(Debug, Clone, Hash, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
/// An artifact lint error or warning
pub struct Lint {
    pub level: Level,
    pub path: Option<PathBuf>,
    pub line: Option<u64>,
    pub category: Category,
    pub msg: String,
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
}

pub fn io_error<P: AsRef<Path>>(lints: &Sender<Lint>, path: P, err: &str) {
    lints
        .send(Lint {
            level: Level::Error,
            category: Category::LoadPaths,
            path: Some(path.as_ref().to_path_buf()),
            line: None,
            msg: format!("Error during loading: {}", err),
        })
        .expect("failed to send io-error");
}
