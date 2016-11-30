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
// Traits
pub use std::io::Write;
pub use std::fmt::Write as FmtWrite;
pub use std::iter::FromIterator;

// stdlib
pub use std::collections::{HashSet, HashMap};
pub use std::process::exit;
pub use std::path::{Path, PathBuf};

// string processing
pub use std::io;
pub use ansi_term::Style;
pub use ansi_term::Colour::{Red, Blue, Green, Yellow};
pub use regex::{Regex, RegexBuilder};

// cmdline
pub use clap::{Arg, App, SubCommand, ArgMatches, AppSettings as AS, Result as ClapResult};

// module types
pub use super::super::core;
pub use super::super::core::utils;
pub use super::super::core::{
    Project,
    Settings, Artifact, Artifacts,
    ArtType, Loc,
    ArtName, ArtNameRc, ArtNames,
    LoadFromStr};
pub use super::super::ui;
pub use super::super::ui::{FmtSettings, FmtArtifact, PercentSearch, SearchSettings};

pub const COLOR: AS = AS::ColorAuto;
