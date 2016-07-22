
use std::iter::FromIterator;
use std::collections::HashSet;

use regex::Regex;

use core::{Artifact, ArtName, parse_names, Settings};

lazy_static!{
    pub static ref VALID_SEARCH_FIELDS: HashSet<char> = HashSet::from_iter(
        ['N', 'D', 'P', 'O', 'L', 'R', 'T', 'A'].iter().map(|s| s.clone()));
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SearchSettings {
    pub name: bool,
    pub path: bool,
    pub parts: bool,
    pub partof: bool,
    pub loc: bool,
    pub refs: bool,
    pub text: bool,
}

/// SPC-ui-filter
pub fn show_artifact(name: &ArtName,
                       art: &Artifact,
                       pat: &Regex,
                       pat_case: &Regex,
                       search_settings: &SearchSettings)
                       -> bool {
    if search_settings.name {
        if !pat_case.is_match(&name.raw) {
            return false;
        }
    }
    return true;
}
