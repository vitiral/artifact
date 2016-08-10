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
pub use ansi_term::Colour::{Red, Blue, Green, Yellow};
pub use regex::{Regex, RegexBuilder};

// cmdline
pub use clap::{Arg, App, SubCommand, ArgMatches, AppSettings as AS, Result as ClapResult};

// module types
pub use super::super::core;
pub use super::super::core::{
    Settings, Artifact, Artifacts,
    ArtType, Loc,
    ArtName, ArtNameRc, ArtNames,
    LoadFromStr};
pub use super::super::ui;
pub use super::super::ui::{FmtSettings, FmtArtifact, PercentSearch, SearchSettings};

