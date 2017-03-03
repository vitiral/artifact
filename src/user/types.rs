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
//! user: loading and saving user data
//!
//! This module encapsulates the loading and saving of artifacts

/// User options for an `Artifact`
#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct UserArtifact {
    pub partof: Option<String>,
    pub text: Option<String>,
    pub done: Option<String>,
}

/// User options for Settings
#[derive(Debug, Clone, PartialEq, RustcEncodable, RustcDecodable)]
pub struct RawSettings {
    pub artifact_paths: Option<Vec<String>>,
    pub exclude_artifact_paths: Option<Vec<String>>,
    pub code_paths: Option<Vec<String>>,
    pub exclude_code_paths: Option<Vec<String>>,
}
