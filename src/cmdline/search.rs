
use std::fmt::Write;
use std::iter::FromIterator;
use std::collections::HashSet;

use regex::Regex;


use core::{Artifact, ArtName, parse_names, load_toml, Settings};

lazy_static!{
    pub static ref VALID_SEARCH_FIELDS: HashSet<char> = HashSet::from_iter(
        ['N', 'D', 'P', 'O', 'L', 'R', 'T', 'A'].iter().map(|s| s.clone()));
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct PercentSearch {
    pub lt: bool,
    pub perc: u8,
}

#[derive(Debug, Default, PartialEq, Eq)]
pub struct SearchSettings {
    pub use_regex: bool,
    pub name: bool,
    pub path: bool,
    pub parts: bool,
    pub partof: bool,
    pub loc: bool,
    pub refs: bool,
    pub text: bool,
    pub completed: PercentSearch,
    pub tested: PercentSearch,
}

impl SearchSettings {
    pub fn new() -> SearchSettings {
        SearchSettings {
            use_regex: false,
            name: false,
            path: false,
            parts: false,
            partof: false,
            loc: false,
            refs: false,
            text: false,
            completed: PercentSearch{lt: false, perc: 0},
            tested: PercentSearch{lt: false, perc: 0},
        }
    }

    pub fn from_str(s: &str) -> Result<SearchSettings, String> {
        let pattern = HashSet::from_iter(s.chars());
        debug!("got search pattern: {:?}", pattern);
        let invalid: HashSet<char> = pattern.difference(&VALID_SEARCH_FIELDS)
                                            .cloned()
                                            .collect();
        if invalid.len() > 0 {
            let mut msg = String::new();
            write!(msg, "Unknown search fields in pattern: {:?}", invalid).unwrap();
            return Err(msg);
        }
        let mut set = SearchSettings {
            use_regex: true,
            name: pattern.contains(&'N'),
            path: pattern.contains(&'D'),
            parts: pattern.contains(&'P'),
            partof: pattern.contains(&'O'),
            loc: pattern.contains(&'L'),
            refs: pattern.contains(&'R'),
            text: pattern.contains(&'T'),
            completed: PercentSearch{lt: false, perc: 0},
            tested: PercentSearch{lt: false, perc: 0},
        };
        if pattern.contains(&'A') {
            set.name = !set.name;
            set.path = !set.path;
            set.parts = !set.parts;
            set.partof = !set.partof;
            set.loc = !set.loc;
            set.refs = !set.refs;
            set.text = !set.text;
        }
        Ok(set)
    }
}
fn matches_name(pat: &Regex, names: &HashSet<ArtName>) -> bool {
    for n in names.iter() {
        if pat.is_match(&n.raw) {
            return true;
        }
    }
    false
}

/// #SPC-ui-filter
pub fn show_artifact(name: &ArtName,
                     art: &Artifact,
                     pat_case: &Regex,
                     search_settings: &SearchSettings)
                     -> bool {
    let ss = search_settings;
    let completed = (art.completed * 100.0).round() as u8;
    let tested = (art.tested * 100.0).round() as u8;
    if (ss.completed.lt && completed > ss.completed.perc)
        || (!ss.completed.lt && completed < ss.completed.perc)
        || (ss.tested.lt && tested > ss.tested.perc)
        || (!ss.tested.lt && tested < ss.tested.perc) {
        false
    } else if !ss.use_regex {
        true
    } else if (ss.name && pat_case.is_match(&name.raw))
        || (ss.parts && matches_name(pat_case, &art.parts))
        || (ss.partof && matches_name(pat_case, &art.partof))
        || (ss.loc && match art.loc.as_ref() {
             None => false,
             Some(l) => pat_case.is_match(l.path.to_string_lossy().as_ref()),
        })
        || (ss.refs && art.refs.iter().any(|r| pat_case.is_match(r)))
        || (ss.text && pat_case.is_match(&art.text)) {
        true
    } else {
        false
    }
}

#[test]
/// [#TST-ui-filter]
fn test_show_artfact() {
    let mut req_one = Artifact::from_str("[REQ-one]
            partof = 'REQ-base'
            text = 'hello bob'").unwrap();
    let mut req_two = Artifact::from_str("[REQ-two]\ntext = 'goodbye joe'").unwrap();
    req_one.1.tested = 0.2;
    req_one.1.completed = 0.8;

    let search_bob = &Regex::new("bob").unwrap();
    let search_two = &Regex::new("two").unwrap();

    // test percentage search
    let mut settings_little_tested = SearchSettings::new();
    settings_little_tested.tested = PercentSearch{lt: false, perc: 10};
    assert!(show_artifact(&req_one.0, &req_one.1, &search_bob, &settings_little_tested));

    let mut settings_ct = SearchSettings::new();
    settings_ct.completed = PercentSearch{lt: false, perc: 50};
    settings_ct.tested = PercentSearch{lt: false, perc: 50};
    assert!(!show_artifact(&req_one.0, &req_one.1, &search_bob, &settings_ct));

    // test regex search
    let settings_name = SearchSettings::from_str("N").unwrap();
    let settings_text = SearchSettings::from_str("T").unwrap();
    let settings_nt = SearchSettings::from_str("NT").unwrap();

    assert!(show_artifact(&req_one.0, &req_one.1, &search_bob, &settings_text));
    assert!(show_artifact(&req_one.0, &req_one.1, &search_bob, &settings_nt));
    assert!(show_artifact(&req_two.0, &req_two.1, &search_two, &settings_nt));

    assert!(!show_artifact(&req_one.0, &req_one.1, &search_bob, &settings_name));
    assert!(!show_artifact(&req_one.0, &req_one.1, &search_two, &settings_nt));
}

