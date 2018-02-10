
use dev_prefix::*;
use types::*;

use ui::types::*;

//#[cfg(test)]
//use user::types::*;

lazy_static!{
    pub static ref VALID_SEARCH_FIELDS: HashSet<String> = HashSet::from_iter(
        ["N", "D", "P", "O", "L", "R", "T", "A",
        "name", "def", "parts", "partof", "loc", "recurse", "text", "all"]
        .iter().map(|s| s.to_string()));
}

impl FromStr for SearchSettings {
    type Err = Error;

    fn from_str(s: &str) -> Result<SearchSettings> {
        let s = s.replace(' ', "");
        if s.is_empty() {
            let set = SearchSettings {
                use_regex: true,
                name: true,
                text: true,
                ..SearchSettings::default()
            };
            debug!("Using default search pattern: {:?}", set);
            return Ok(set);
        }

        let first_char = s.chars().next().unwrap();
        let pattern: HashSet<String> = match first_char {
            'a'...'z' => s.split(',').map(|s| s.to_string()).collect(),
            _ => s.chars().map(|c| c.to_string()).collect(),
        };

        debug!("Got search pattern: {:?}", pattern);
        let invalid: HashSet<String> = pattern.difference(&VALID_SEARCH_FIELDS).cloned().collect();
        if !invalid.is_empty() {
            return Err(
                ErrorKind::CmdError(format!("Unknown search fields in pattern: {:?}", invalid))
                    .into(),
            );
        }
        let pc = |c| pattern.contains(c);
        let mut set = SearchSettings {
            use_regex: true,
            name: pc("N") || pc("name"),
            def: pc("D") || pc("def"),
            parts: pc("P") || pc("parts"),
            partof: pc("O") || pc("partof"),
            loc: pc("L") || pc("loc"),
            text: pc("T") || pc("text"),
            ..SearchSettings::default()
        };
        if pc("A") || pc("all") {
            set.name = !set.name;
            set.def = !set.def;
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
pub fn show_artifact(
    name: &Name,
    art: &Artifact,
    pat_case: &Regex,
    search_settings: &SearchSettings,
) -> bool {
    let ss = search_settings;
    let completed = (art.completed * 100.0).round() as i8;
    let tested = (art.tested * 100.0).round() as i8;
    if (ss.completed.lt && completed > ss.completed.perc) ||
        (!ss.completed.lt && completed < ss.completed.perc) ||
        (ss.tested.lt && tested > ss.tested.perc) ||
        (!ss.tested.lt && tested < ss.tested.perc)
    {
        false
    } else {
        !ss.use_regex || (ss.name && pat_case.is_match(&name.raw)) ||
            (ss.parts && matches_name(pat_case, &art.parts)) ||
            (ss.partof && matches_name(pat_case, &art.partof)) ||
            (ss.loc && match art.done {
                Done::Code(ref l) => if let Some(ref l) = l.root {
                    pat_case.is_match(l.path.to_string_lossy().as_ref())
                } else {
                    false
                },
                Done::Defined(ref s) => pat_case.is_match(s),
                Done::NotDone => false,
            }) || (ss.text && pat_case.is_match(&art.text))
    }
}

#[test]
fn test_search_settings() {
    assert_eq!(
        SearchSettings::from_str("NT").unwrap(),
        SearchSettings::from_str("").unwrap()
    );

    let fs = |s| SearchSettings::from_str(s).unwrap();
    assert_eq!(
        fs("NT"),
        SearchSettings {
            use_regex: true,
            name: true,
            text: true,
            ..SearchSettings::default()
        }
    );
    assert_eq!(fs("NDPL"), fs("name, def, parts, loc"));

    // build it up one at a time
    {
        let mut set = SearchSettings::default();
        set.use_regex = true;
        set.name = true;
        assert_eq!(set, fs("N"));
        set.parts = true;
        assert_eq!(set, fs("NP"));
        set.def = true;
        assert_eq!(set, fs("NPD"));
        set.partof = true;
        assert_eq!(set, fs("NPDO"));
        set.text = true;
        assert_eq!(set, fs("NPDOT"));
        set.loc = true;
        assert_eq!(set, fs("NPDOTL"));
    }

    assert!(SearchSettings::from_str("foobar").is_err());
}

#[test]
fn test_show_artifact() {
    let mut req_one = Artifact::from_str(
        "[REQ-one]
            partof = 'REQ-base'
            text = 'hello bob'",
    ).unwrap();
    let req_two = Artifact::from_str("[REQ-two]\ntext = 'goodbye joe'").unwrap();
    req_one.1.tested = 0.2;
    req_one.1.completed = 0.8;

    let search_bob = &Regex::new("bob").unwrap();
    let search_two = &Regex::new("two").unwrap();

    // test percentage search
    let mut settings_little_tested = SearchSettings::default();
    settings_little_tested.tested = PercentSearch {
        lt: false,
        perc: 10,
    };
    assert!(show_artifact(
        &req_one.0,
        &req_one.1,
        &search_bob,
        &settings_little_tested,
    ));

    let mut settings_ct = SearchSettings::default();
    settings_ct.completed = PercentSearch {
        lt: false,
        perc: 50,
    };
    settings_ct.tested = PercentSearch {
        lt: false,
        perc: 50,
    };
    assert!(!show_artifact(
        &req_one.0,
        &req_one.1,
        &search_bob,
        &settings_ct,
    ));

    // test regex search
    let settings_name = SearchSettings::from_str("N").unwrap();
    let settings_text = SearchSettings::from_str("T").unwrap();
    let settings_nt = SearchSettings::from_str("NT").unwrap();

    assert!(show_artifact(
        &req_one.0,
        &req_one.1,
        &search_bob,
        &settings_text,
    ));
    assert!(show_artifact(
        &req_one.0,
        &req_one.1,
        &search_bob,
        &settings_nt,
    ));
    assert!(show_artifact(
        &req_two.0,
        &req_two.1,
        &search_two,
        &settings_nt,
    ));

    assert!(!show_artifact(
        &req_one.0,
        &req_one.1,
        &search_bob,
        &settings_name,
    ));
    assert!(!show_artifact(
        &req_one.0,
        &req_one.1,
        &search_two,
        &settings_nt,
    ));
}
