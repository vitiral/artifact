
use super::super::types::*;
use super::super::matches::*;
use super::super::ls;

use std::thread;
use std::time;


#[test]
/// partof: #TST-ls-interface
fn test_get_matches() {
    let args = vec!["rst", "ls", "-l"];
    let matches = get_matches(&args).unwrap();
    let (search, fmtset, search_set) = ls::get_ls_cmd(matches.subcommand_matches("ls").unwrap())
                                           .unwrap();
    assert_eq!(search, "");
    assert_eq!(fmtset.long, true);
    assert_eq!(fmtset.recurse, 0);
    assert_eq!(search_set, SearchSettings::new());

    // test that -A works
    let args = vec!["rst", "ls", "all", "-AP"];
    let matches = get_matches(&args).unwrap();
    let (search, fmtset, search_set) = ls::get_ls_cmd(matches.subcommand_matches("ls").unwrap())
                                           .unwrap();
    assert_eq!(search, "all");
    assert_eq!(fmtset.long, false);
    assert_eq!(fmtset.parts, false);
    assert_eq!(fmtset.partof, true);
    assert_eq!(fmtset.loc_path, true);
    assert_eq!(fmtset.recurse, 0);
    assert_eq!(search_set, SearchSettings::new());

    // test that pattern works
    // #TST-ls-search
    let args = vec!["rst", "ls", "regex", "-p", "TNL"];
    let matches = get_matches(&args).unwrap();
    let (search, _, search_set) = ls::get_ls_cmd(matches.subcommand_matches("ls").unwrap())
                                      .unwrap();
    assert_eq!(search, "regex");
    assert!(search_set.text);
    assert!(search_set.name);
    assert!(search_set.loc);
}


#[test]
fn test_ls() {
    let (mut fmt_set, mut search_set, mut settings) = (FmtSettings::default(),
                                                       SearchSettings::default(),
                                                       Settings::default());
    let mut artifacts = core::load::load_toml_simple("[REQ-foo]\n[SPC-foo]\n[TST-foo]\n");
    core::link::link_named_partofs(&mut artifacts);
    core::link::link_parents(&mut artifacts);
    core::link::validate_partof(&artifacts).unwrap();
    core::link::link_parts(&mut artifacts);
    fmt_set.color = true;
    let mut w: Vec<u8> = Vec::new();
    let cwd = PathBuf::from("src/foo");
    let reqs_path = PathBuf::from("reqs/foo.toml");
    for (_, a) in artifacts.iter_mut() {
        a.path = reqs_path.clone();
        a.completed = 1.;
        a.tested = 0.5;
    }
    ls::do_ls(&mut w,
              &cwd,
              "req-foo", // case does not matter
              &artifacts,
              &fmt_set,
              &search_set,
              &settings);
    fn vb(b: &'static [u8]) -> Vec<u8> {
        Vec::from_iter(b.iter().cloned())
    }
    /// if the format changes, you can use this to help create the test for color
    /// just pass it in and copy-paste (validating that it looks right first of course...)
    #[allow(dead_code)]
    fn debug_bytes(bytes: &Vec<u8>) {
        // sleep for a bit so stderr passes us
        thread::sleep(time::Duration::new(0, 2e8 as u32));
        println!("Debug:");
        for b in bytes {
            print!("{}", *b as char);
        }
        for b in bytes {
            match *b {
                // 9 => print!("{}", *b as char), // TAB
                b'\n' => print!("\\n"),
                b'\r' => print!("\\r"),
                32...126 => print!("{}", *b as char), // visible ASCII
                _ => print!(r"\x{:0>2x}", b),

            }
        }
        println!("");
    }
    // debug_bytes(&w);
    let expected = b"\x1b[1m|  | DONE TEST | ARTIFACT NAME                                 | PARTS   | DEFINED   \n\x1b[0m|\x1b[1;34mD\x1b[0m\x1b[1;33m-\x1b[0m| \x1b[1;34m100\x1b[0m%  \x1b[1;33m50\x1b[0m% | \x1b[1;4;34mreq-foo\x1b[0m                                       | \x1b[34mSPC-foo\x1b[0m | ../../reqs/foo.toml \n";
    assert_eq!(vb(expected), w);
}
