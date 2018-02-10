use std::thread;
use std::time;

use std::path::MAIN_SEPARATOR;

use dev_prefix::*;
use types::*;
use cmd::types::*;
use user;
use cmd::matches;
use cmd::ls;
use cmd::check;

use test_data;


#[test]
fn test_get_matches() {
    let args = vec!["artifact", "ls", "-l"];
    let matches = matches::get_matches(&args).unwrap();
    let cmd = ls::get_cmd(matches.subcommand_matches("ls").unwrap()).unwrap();
    assert_eq!(cmd.pattern, "");
    assert_eq!(cmd.fmt_settings.long, true);
    assert_eq!(cmd.fmt_settings.recurse, 0);
    assert_eq!(cmd.search_settings, SearchSettings::default());

    // test that -A works
    let args = vec!["artifact", "ls", "all", "-AP"];
    let matches = matches::get_matches(&args).unwrap();
    let cmd = ls::get_cmd(matches.subcommand_matches("ls").unwrap()).unwrap();
    assert_eq!(cmd.pattern, "all");
    assert_eq!(cmd.fmt_settings.long, false);
    assert_eq!(cmd.fmt_settings.parts, false);
    assert_eq!(cmd.fmt_settings.partof, true);
    assert_eq!(cmd.fmt_settings.loc_path, true);
    assert_eq!(cmd.fmt_settings.recurse, 0);
    assert_eq!(cmd.search_settings, SearchSettings::default());

    // test that pattern works
    let args = vec!["artifact", "ls", "regex", "-p", "TNL"];
    let matches = matches::get_matches(&args).unwrap();
    let cmd = ls::get_cmd(matches.subcommand_matches("ls").unwrap()).unwrap();
    assert_eq!(cmd.pattern, "regex");
    assert!(cmd.search_settings.text);
    assert!(cmd.search_settings.name);
    assert!(cmd.search_settings.loc);
}


fn to_vec(b: &'static [u8]) -> Vec<u8> {
    Vec::from_iter(b.iter().map(|c| match *c {
        b'/' => MAIN_SEPARATOR as u8,
        _ => c.clone(),
    }))
}


const ARTIFACT_TEXT: &'static str = r"
[REQ-foo]
text = 'req for foo'
[SPC-foo]
text = 'spc for foo'
[TST-foo]
text = 'tst for foo'
[TST-foo_bar]
partof = 'SPC-foo'
text = 'tst for foo_bar'
[TST-line]
[TST-line-long]
text = 'This line is very very very very long and it should probably get trimmed down.'
[TST-line-multi]
text = '''
This text has multiple lines.
This is the second one.
You shouldn't see these later lines!
'''

[SPC-unresolvable]
partof = 'SPC-unresolvable-1-1'
[SPC-unresolvable-1]
[SPC-unresolvable-1-1]

[REQ-invalid]
[REQ-invalid-parts]
partof = 'REQ-dne'
";


const LS_SPC_DNE_NC: &'static [u8] = b"\nFound partof names that do not exist:\n\
- REQ-invalid-parts [../../reqs/foo.toml]: {REQ-DNE}\n\nArtifacts partof contain\
s at least one recursive reference:\n- SPC-unresolvable              : [SPC-UNRE\
SOLVABLE-1-1]\n- SPC-unresolvable-1            : [SPC-UNRESOLVABLE]\n- SPC-unres\
olvable-1-1          : [SPC-UNRESOLVABLE-1]\n\nFound implementation links in th\
e code that do not exist:\n- ../../fake:\n  - [42] SPC-dne\n\nHanging artifact\
s found (top-level but not partof a higher type):\n- ../../reqs/foo.tom\
l           : TST-line\n\n";

#[cfg(windows)]
const LS_SPC_DNE: &'static [u8] = LS_SPC_DNE_NC;

#[cfg(not(windows))]
const LS_SPC_DNE: &'static [u8] = b"\x1b[1;31m\nFound partof names that do not e\
xist:\n\x1b[0m\x1b[31m- REQ-invalid-parts [../../reqs/foo.toml]: {REQ-DNE}\n\
\x1b[0m\x1b[1;31m\nArtifacts partof contains at least one recursive reference:\n\
\x1b[0m- SPC-unresolvable              : [SPC-UNRESOLVABLE-1-1]\n- SPC-unresolva\
ble-1            : [SPC-UNRESOLVABLE]\n- SPC-unresolvable-1-1          : [SPC-UN\
RESOLVABLE-1]\n\x1b[1;31m\nFound implementation links in the code that do not ex\
ist:\n\x1b[0m\x1b[31m- ../../fake:\n\x1b[0m\x1b[31m  - [42]\x1b[0m SPC-dne\n\
\x1b[1;31m\nHanging artifacts found (top-level but not partof a higher type):\n\
\x1b[0m- ../../reqs/foo.toml           : TST-line\n\n";

const LS_REQ_FOO_NC: &'static [u8] =
    b"|  | DONE TEST | NAME     | PARTS   \n|D-| 100%  50% | req-foo  | SPC-foo\n";

#[cfg(not(windows))]
const LS_REQ_FOO: &'static [u8] = b"\x1b[1m|  | DONE TEST | NAME      | PARTS   \n\x1b[0m|\x1b\
[1;34mD\x1b[0m\x1b[1;33m-\x1b[0m| \x1b[1;34m100\x1b[0m%  \x1b[1;33m50\x1b[0m% | \x1b[1;4;34mreq-foo\
\x1b[0m   | \x1b[34mSPC-foo\x1b[0m\n";

#[cfg(windows)]
const LS_REQ_FOO: &'static [u8] = LS_REQ_FOO_NC;


const LS_S_C_STAR_FOO_NC: &'static [u8] = b"|  | DONE TEST | NAME     | \
PARTS     | PARTOF     | IMPLEMENTED     | DEFINED              | TEXT\n|D-| \
100%  50% | REQ-foo  | SPC-foo   |            |                 | ../../reqs/foo\
.toml  | req for foo\n";

#[cfg(not(windows))]
const LS_S_C_STAR_FOO: &'static [u8] = b"\x1b[1m|  | DONE TEST | NAME      | \
PARTS     | PARTOF     | IMPLEMENTED     | DEFINED              | TEXT\n\
\x1b[0m|\x1b[1;34mD\x1b[0m\x1b[1;33m-\x1b[0m| \x1b[1;34m100\x1b[0m%  \
\x1b[1;33m50\x1b[0m% | \x1b[1;4;34mREQ-foo\x1b[0m   | \x1b[34mSPC-foo\x1b[0m   \
|            | \x1b[32m\x1b[0m                | ../../reqs/foo.toml  | req for \
foo\n";

#[cfg(windows)]
const LS_S_C_STAR_FOO: &'static [u8] = LS_S_C_STAR_FOO_NC;

const LS_T_LONG_NC: &'static [u8] = b"|  | DONE TEST | NAME           | TEXT\n|--|   0%   0% | \
TST-line-long  | This line is very very very very long and it sh...\n";

#[cfg(not(windows))]
const LS_T_LONG: &'static [u8] = b"\x1b[1m|  | DONE TEST | NAME            | \
 TEXT\n\x1b[0m|\x1b[1;31m-\x1b[0m\x1b[1;31m-\x1b[0m|   \x1b[1;31m0\x1b[0m%   \
 \x1b[1;31m0\x1b[0m% | \x1b[1;4;31mTST-line-long\x1b[0m   | \
 This line is very very very very long and it sh...\n";

#[cfg(windows)]
const LS_T_LONG: &'static [u8] = LS_T_LONG_NC;


const LS_T_MULTI_NC: &'static [u8] = b"|  | DONE TEST | NAME            | TEXT\n|--|   0%   0% | \
TST-line-multi  | This text has multiple lines.\n";

#[cfg(not(windows))]
const LS_T_MULTI: &'static [u8] = b"\x1b[1m|  | DONE TEST | NAME             | \
TEXT\n\x1b[0m|\x1b[1;31m-\x1b[0m\x1b[1;31m-\x1b[0m|   \x1b[1;31m0\x1b[0m%   \
\x1b[1;31m0\x1b[0m% | \x1b[1;4;31mTST-line-multi\x1b[0m   | \
This text has multiple lines.\n";

#[cfg(windows)]
const LS_T_MULTI: &'static [u8] = LS_T_MULTI_NC;

const LS_L_MULTI_NC: &'static [u8] = b"|--|   0%   0% | TST-line-multi\n * text:\nThis text has \
multiple lines.\nThis is the second one.\nYou shouldn't see these later lines!\n\n";

#[cfg(not(windows))]
const LS_L_MULTI: &'static [u8] = b"|\x1b[1;31m-\x1b[0m\x1b[1;31m-\x1b[0m|   \
\x1b[1;31m0\x1b[0m%   \x1b[1;31m0\x1b[0m% | \x1b[1;4;31mTST-line-multi\x1b[0m \
\x1b[32m\n * text:\n\x1b[0mThis text has multiple lines.\nThis is the second one.\n\
You shouldn't see these later lines!\n\n";

#[cfg(windows)]
const LS_L_MULTI: &'static [u8] = LS_L_MULTI_NC;

const LS_FILTER: &'static [u8] =
    b"|  | DONE TEST | NAME     | PARTS   \n|DT| 100% 100% | TST-foo  | \n";

#[cfg(not(windows))]
const COLOR_IF_POSSIBLE: bool = true;

#[cfg(windows)]
const COLOR_IF_POSSIBLE: bool = false;


fn repr_bytes(bytes: &[u8]) {
    for b in bytes {
        match *b {
            // 9 => print!("{}", *b as char), // TAB
            b'\n' => print!("\\n"),
            b'\r' => print!("\\r"),
            32...126 => print!("{}", *b as char), // visible ASCII
            _ => print!(r"\x{:0>2x}", b),
        }
    }
}

/// if the format changes, you can use this to help create the test for color
/// just pass it in and copy-paste (validating that it looks right first of course...)
#[allow(dead_code)]
fn debug_bytes(result: &[u8], expected: &[u8]) {
    // sleep for a bit so stderr passes us
    thread::sleep(time::Duration::new(0, 2e8 as u32));
    println!("\nDebug Result:");
    for b in result {
        print!("{}", *b as char);
    }
    println!("Repr Result:");
    repr_bytes(result);
    println!("");
    println!("--Result Repr DONE");

    println!("Debug Expected:");
    for b in expected {
        print!("{}", *b as char);
    }
    println!("Repr Expected:");
    repr_bytes(expected);
    println!("");
    println!("--Expected Repr DONE\n");
}

fn get_project() -> Project {
    let mut artifacts = test_data::load_toml_simple(ARTIFACT_TEXT);
    let reqs_path = PathBuf::from("reqs/foo.toml");
    for (n, a) in artifacts.iter_mut() {
        a.def = reqs_path.clone();
        if n.as_ref() == &Name::from_str("spc-foo").unwrap() {
            a.done = Done::Code(FullLocs::fake());
        }
        if n.as_ref() == &Name::from_str("tst-foo").unwrap() {
            a.done = Done::Code(FullLocs::fake());
        }
    }
    user::do_links(&mut artifacts).unwrap();
    let dne_locs: HashMap<_, _> =
        HashMap::from_iter(vec![(Name::from_str("SPC-dne").unwrap(), Loc::fake())]);
    let mut project = Project::default();
    project.artifacts = artifacts;
    project.dne_locs = dne_locs;
    project
}


#[test]
fn test_cmd_check() {
    let mut cmd = check::Cmd {
        color: COLOR_IF_POSSIBLE,
    };
    let mut w: Vec<u8> = Vec::new();
    let cwd = PathBuf::from("src/foo");
    let project = get_project();

    // #TST-cmd-check
    w.clear();
    assert!(check::run_cmd(&mut w, &cwd, &project, &cmd).is_err());
    debug_bytes(&w, LS_SPC_DNE);
    assert_eq!(to_vec(LS_SPC_DNE), w);

    w.clear();
    cmd.color = false;
    assert!(check::run_cmd(&mut w, &cwd, &project, &cmd).is_err());
    debug_bytes(&w, LS_SPC_DNE_NC);
    assert_eq!(to_vec(LS_SPC_DNE_NC), w);
}

#[test]
/// #TST-cmd-ls
fn test_cmd_ls() {
    let mut cmd = ls::Cmd {
        pattern: "".to_string(),
        fmt_settings: FmtSettings::default(),
        search_settings: SearchSettings::default(),
        ty: ls::OutType::List,
    };
    cmd.fmt_settings.color = COLOR_IF_POSSIBLE;
    let mut w: Vec<u8> = Vec::new();
    let cwd = PathBuf::from("src/foo");

    let project = get_project();

    // do default list, looking for only req-foo
    w.clear();
    cmd.fmt_settings.color = COLOR_IF_POSSIBLE;
    cmd.pattern = "req-foo".to_string();
    ls::run_cmd(&mut w, &cwd, &cmd, &project).unwrap();
    debug_bytes(&w, LS_REQ_FOO);
    assert_eq!(to_vec(LS_REQ_FOO), w);

    // do default list with color disabled
    w.clear();
    cmd.fmt_settings.color = false;
    ls::run_cmd(&mut w, &cwd, &cmd, &project).unwrap();
    //debug_bytes(&w, LS_REQ_FOO_NC);
    assert_eq!(to_vec(LS_REQ_FOO_NC), w);

    // default list for non-existant requirement
    w.clear();
    cmd.pattern = "REQ_DNE".to_string();
    assert!(ls::run_cmd(&mut w, &cwd, &cmd, &project).is_err());
    assert_eq!(to_vec(b""), w);

    // Test that if the first line is too long that it's trimmed
    w.clear();
    cmd.pattern = "TST-line-long".to_string();
    cmd.fmt_settings.color = COLOR_IF_POSSIBLE;
    cmd.fmt_settings.text = true;
    ls::run_cmd(&mut w, &cwd, &cmd, &project).unwrap();
    debug_bytes(&w, LS_T_LONG);
    assert_eq!(to_vec(LS_T_LONG), w);

    w.clear();
    cmd.fmt_settings.color = false;
    ls::run_cmd(&mut w, &cwd, &cmd, &project).unwrap();
    //debug_bytes(&w, LS_T_LONG_NC);
    assert_eq!(to_vec(LS_T_LONG_NC), w);

    // Test that only the first line is selected
    w.clear();
    cmd.fmt_settings.color = COLOR_IF_POSSIBLE;
    cmd.pattern = "TST-line-multi".to_string();
    cmd.fmt_settings.text = true;
    ls::run_cmd(&mut w, &cwd, &cmd, &project).unwrap();
    // debug_bytes(&w, LS_T_MULTI);
    assert_eq!(to_vec(LS_T_MULTI), w);

    w.clear();
    cmd.fmt_settings.color = false;
    ls::run_cmd(&mut w, &cwd, &cmd, &project).unwrap();
    //debug_bytes(&w, LS_T_MULTI_NC);
    assert_eq!(to_vec(LS_T_MULTI_NC), w);

    // Test that -l output looks correct
    w.clear();
    cmd.fmt_settings.color = COLOR_IF_POSSIBLE;
    cmd.pattern = "TST-line-multi".to_string();
    cmd.fmt_settings.text = true;
    cmd.fmt_settings.long = true;
    ls::run_cmd(&mut w, &cwd, &cmd, &project).unwrap();
    // debug_bytes(&w, LS_L_MULTI);
    assert_eq!(to_vec(LS_L_MULTI), w);

    w.clear();
    cmd.fmt_settings.color = false;
    ls::run_cmd(&mut w, &cwd, &cmd, &project).unwrap();
    //debug_bytes(&w, LS_L_MULTI_NC);
    assert_eq!(to_vec(LS_L_MULTI_NC), w);

    // ls all fields
    // do a search in only parts using regex s.c
    w.clear();
    cmd.pattern = "s.c.*foo".to_string();
    cmd.fmt_settings.long = false;
    cmd.fmt_settings.color = COLOR_IF_POSSIBLE;
    cmd.fmt_settings.def = true;
    cmd.fmt_settings.parts = true;
    cmd.fmt_settings.partof = true;
    cmd.fmt_settings.loc_path = true;
    cmd.fmt_settings.text = true;
    cmd.search_settings.use_regex = true;
    cmd.search_settings.parts = true;
    ls::run_cmd(&mut w, &cwd, &cmd, &project).unwrap();
    debug_bytes(&w, LS_S_C_STAR_FOO);
    assert_eq!(to_vec(LS_S_C_STAR_FOO), w);

    w.clear();
    cmd.fmt_settings.color = false;
    ls::run_cmd(&mut w, &cwd, &cmd, &project).unwrap();
    debug_bytes(&w, LS_S_C_STAR_FOO_NC);
    assert_eq!(to_vec(LS_S_C_STAR_FOO_NC), w);

    // test filtering
    w.clear();
    cmd.pattern = "".to_string();
    cmd.fmt_settings = FmtSettings::default();
    cmd.search_settings = SearchSettings::default();

    cmd.fmt_settings.color = false;
    cmd.search_settings.completed = PercentSearch {
        lt: false,
        perc: 100,
    };
    cmd.search_settings.tested = PercentSearch {
        lt: false,
        perc: 100,
    };
    ls::run_cmd(&mut w, &cwd, &cmd, &project).unwrap();
    debug_bytes(&w, LS_FILTER);
    assert_eq!(to_vec(LS_FILTER), w);
}
