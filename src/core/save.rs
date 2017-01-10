/*  rst: the requirements tracking tool made for developers
    Copyright (C) 2016  Garrett Berg <@vitiral, vitiral@gmail.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the Lesser GNU General Public License as published 
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the Lesser GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/
//! loadrs
//! loading of raw artifacts from files and text

use rustc_serialize::{Decodable};
use toml::{encode, Parser, Value, Table, Decoder};

use dev_prefix::*;
use super::types::*;
use super::vars;
use super::utils;
use super::load;

/// save a project to it's files
/// see: #SPC-save
pub fn save_project(project: &mut Project) -> Result<()> {
    let mut files: HashMap<PathBuf, Table> = HashMap::new();

    // we just go through each item, growing `files` as necessary
    for (path, raw) in &project.raw_settings_map {
        // insert settings
        files.insert(path.clone(), Table::new());
        let tbl = files.get_mut(path).unwrap();
        tbl.insert("settings".to_string(), encode(raw));
    }
    for (path, raw) in &project.variables_map {
        // insert variables (globals)
        if !files.contains_key(path) {
            files.insert(path.clone(), Table::new());
        }
        let tbl = files.get_mut(path).unwrap();
        tbl.insert("globals".to_string(), encode(raw));
    }
    for (name, artifact) in &project.artifacts {
        // insert artifacts
        if !files.contains_key(&artifact.path) {
            files.insert(artifact.path.clone(), Table::new());
        }
        let tbl = files.get_mut(&artifact.path).unwrap();

        let partof = match artifact.partof.is_empty() {
            true => None,
            false => Some(artifact.partof.iter().map(|p| p.raw.clone())
                          .collect::<Vec<_>>()),
        };
        let raw = RawArtifact {
            partof: None, // TODO: use partof
            text: match artifact.text.raw.is_empty() {
                true => Some(artifact.text.raw.clone()),
                false => None,
            },
        };
        tbl.insert(name.raw.clone(), encode(&raw));
    }
    Err(ErrorKind::PathNotFound("woo".to_string()).into())
}

