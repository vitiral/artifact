
use dev_prefix::*;
use types::*;

use ui::types::*;

//#[cfg(test)]
//use user::types::*;

lazy_static!{
    pub static ref VALID_SEARCH_FIELDS: HashSet<char> = HashSet::from_iter(
        ['N', 'D', 'P', 'O', 'L', 'R', 'T', 'A'].iter().cloned());
}

impl FromStr for SearchSettings {
    type Err = Error;

    fn from_str(s: &str) -> Result<SearchSettings> {
        let pattern = HashSet::from_iter(s.chars());
        debug!("got search pattern: {:?}", pattern);
        let invalid: HashSet<char> = pattern.difference(&VALID_SEARCH_FIELDS).cloned().collect();
        if !invalid.is_empty() {
            return Err(ErrorKind::CmdError(format!("Unknown search fields in pattern: {:?}",
                                                   invalid))
                               .into());
        }
        let mut set = SearchSettings {
            use_regex: true,
            name: pattern.contains(&'N'),
            path: pattern.contains(&'D'),
            parts: pattern.contains(&'P'),
            partof: pattern.contains(&'O'),
            loc: pattern.contains(&'L'),
            text: pattern.contains(&'T'),
            ..SearchSettings::default()
        };
        if pattern.contains(&'A') {
            set.name = !set.name;
            set.path = !set.path;
            set.parts = !set.parts;
            set.partof = !set.partof;
            set.loc = !set.loc;
            set.text = !set.text;
        }
        Ok(set)
    }
}

fn matches_name(pat: &Regex, names: &Names) -> bool {
    for n in names.iter() {
        if pat.is_match(&n.raw) {
            return true;
        }
    }
    false
}

/// return true if the artifact meets the criteria
pub fn show_artifact(name: &Name,
                     art: &Artifact,
                     pat_case: &Regex,
                     search_settings: &SearchSettings)
                     -> bool {
    let ss = search_settings;
    let completed = (art.completed * 100.0).round() as i8;
    let tested = (art.tested * 100.0).round() as i8;
    if (ss.completed.lt && completed > ss.completed.perc) ||
       (!ss.completed.lt && completed < ss.completed.perc) ||
       (ss.tested.lt && tested > ss.tested.perc) ||
       (!ss.tested.lt && tested < ss.tested.perc) {
        false
    } else {
        !ss.use_regex || (ss.name && pat_case.is_match(&name.raw)) ||
        (ss.parts && matches_name(pat_case, &art.parts)) ||
        (ss.partof && matches_name(pat_case, &art.partof)) ||
        (ss.loc &&
         match art.done {
             Done::Code(ref l) => pat_case.is_match(l.path.to_string_lossy().as_ref()),
             Done::Defined(ref s) => pat_case.is_match(s),
             Done::NotDone => false,
         }) || (ss.text && pat_case.is_match(&art.text))
    }
}

// FIXME
//#[test]
//fn test_show_artfact() {
//    let mut req_one = Artifact::from_str("[REQ-one]
//            partof = 'REQ-base'
//            text = 'hello bob'")
//        .unwrap();
//    let req_two = Artifact::from_str("[REQ-two]\ntext = 'goodbye joe'").unwrap();
//    req_one.1.tested = 0.2;
//    req_one.1.completed = 0.8;

//    let search_bob = &Regex::new("bob").unwrap();
//    let search_two = &Regex::new("two").unwrap();

//    // test percentage search
//    let mut settings_little_tested = SearchSettings::default();
//    settings_little_tested.tested = PercentSearch {
//        lt: false,
//        perc: 10,
//    };
//    assert!(show_artifact(&req_one.0, &req_one.1, &search_bob, &settings_little_tested));

//    let mut settings_ct = SearchSettings::default();
//    settings_ct.completed = PercentSearch {
//        lt: false,
//        perc: 50,
//    };
//    settings_ct.tested = PercentSearch {
//        lt: false,
//        perc: 50,
//    };
//    assert!(!show_artifact(&req_one.0, &req_one.1, &search_bob, &settings_ct));

//    // test regex search
//    let settings_name = SearchSettings::from_str("N").unwrap();
//    let settings_text = SearchSettings::from_str("T").unwrap();
//    let settings_nt = SearchSettings::from_str("NT").unwrap();

//    assert!(show_artifact(&req_one.0, &req_one.1, &search_bob, &settings_text));
//    assert!(show_artifact(&req_one.0, &req_one.1, &search_bob, &settings_nt));
//    assert!(show_artifact(&req_two.0, &req_two.1, &search_two, &settings_nt));

//    assert!(!show_artifact(&req_one.0, &req_one.1, &search_bob, &settings_name));
//    assert!(!show_artifact(&req_one.0, &req_one.1, &search_two, &settings_nt));
//}
