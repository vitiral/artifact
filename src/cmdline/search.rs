
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

fn matches_name(pat: &Regex, names: &HashSet<ArtName>) -> bool {
    for n in names.iter() {
        if pat.is_match(&n.raw) {
            return true;
        }
    }
    false
}

/// SPC-ui-filter
pub fn show_artifact(name: &ArtName,
                       art: &Artifact,
                       pat_case: &Regex,
                       search_settings: &SearchSettings)
                       -> bool {
    let ss = search_settings;
    if (ss.name && pat_case.is_match(&name.raw))
        || (ss.parts && matches_name(pat_case, &art.parts))
        || (ss.partof && matches_name(pat_case, &art.partof))
        || (ss.loc && match art.loc.as_ref() {
             None => false,
             Some(l) => pat.is_match(l.path.to_string_lossy().as_ref()),
        })
        || (ss.refs && art.refs.iter().any(|r| pat_case.is_match(r)))
        || (ss.text && pat_case.is_match(&art.text)) {
        true
    } else {
        false
    }
}
