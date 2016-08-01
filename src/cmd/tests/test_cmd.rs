
use super::super::types::*;
use super::super::matches::*;
use super::super::ls;

#[test]
/// partof: #TST-ls-interface
fn test_get_matches() {
    let args = vec!["rsk", "ls", "-l"];
    let matches = get_matches(&args).unwrap();
    let (search, fmtset, search_set) = ls::get_ls_cmd(
        matches.subcommand_matches("ls").unwrap()).unwrap();
    assert_eq!(search, "");
    assert_eq!(fmtset.long, true);
    assert_eq!(fmtset.recurse, 0);
    assert_eq!(search_set, SearchSettings::new());

    // test that -A works
    let args = vec!["rsk", "ls", "all", "-AP"];
    let matches = get_matches(&args).unwrap();
    let (search, fmtset, search_set) = ls::get_ls_cmd(
        matches.subcommand_matches("ls").unwrap()).unwrap();
    assert_eq!(search, "all");
    assert_eq!(fmtset.long, false);
    assert_eq!(fmtset.parts, false);
    assert_eq!(fmtset.partof, true);
    assert_eq!(fmtset.loc_path, true);
    assert_eq!(fmtset.recurse, 0);
    assert_eq!(search_set, SearchSettings::new());

    // test that pattern works
    // #TST-ls-search
    let args = vec!["rsk", "ls", "regex", "-p", "TNL"];
    let matches = get_matches(&args).unwrap();
    let (search, _, search_set) = ls::get_ls_cmd(
        matches.subcommand_matches("ls").unwrap()).unwrap();
    assert_eq!(search, "regex");
    assert!(search_set.text);
    assert!(search_set.name);
    assert!(search_set.loc);
}
