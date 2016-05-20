use super::super::load::*;

// Data and helpers

static TOML_TEST: &'static str = "
[settings]
disabled = false
paths = ['{cwd}/test', '{repo}/test']
repo_names = ['.test']

[REQ-foo]
disabled = false
[SPC-foo]
refs = [1, 2]
[RSK-foo]
[TST-foo]
[REQ-bar]
text = 'bar'
disabled = false
refs = [\"hello\", \"ref\"]
";

static TOML_GOOD: &'static str = "
[settings]
disabled = false
paths = ['{cwd}/data/empty']
repo_names = ['.test']

[REQ-foo]
disabled = false
[SPC-foo]
refs = [1, 2]
[RSK-foo]
[TST-foo]
[REQ-bar]
text = 'bar'
disabled = false
refs = [\"hello\", \"ref\"]
";

static TOML_BAD: &'static str = "[REQ-bad]\ndone = '100%'";
static TOML_OVERLAP: &'static str = "[REQ-foo]\n";

fn parse_text(t: &str) -> Table {
    Parser::new(t).parse().unwrap()
}

fn get_table<'a>(tbl: &'a Table, attr: &str) -> &'a Table {
    match tbl.get(attr).unwrap() {
        &Value::Table(ref t) => t,
        _ => unreachable!()
    }
}

// Tests

#[test]
fn test_get_attr() {
    let tbl_good = parse_text(TOML_TEST);
    let df_str = "".to_string();
    let df_tbl = Table::new();
    let ref df_vec: Vec<String> = Vec::new();

    // LOC-tst-core-load-attrs-unit-1:<Test loading valid existing types>
    let test = get_attr!(tbl_good, "REQ-bar", df_tbl, Table).unwrap();
    assert!(get_attr!(&test, "disabled", false, Boolean).unwrap() == false);
    assert!(get_attr!(&test, "disabled", true, Boolean).unwrap() == false);
    assert!(get_attr!(&test, "text", df_str, String).unwrap() == "bar");
    assert!(get_attr!(&test, "text", df_str, String).unwrap() == "bar");
    assert!(get_vecstr(&test, "refs", df_vec).unwrap() == ["hello", "ref"]);

    // LOC-tst-core-load-attrs-unit-2:<Test loading invalid existing types>
    assert!(get_attr!(&test, "disabled", df_str, String).is_none());
    assert!(get_attr!(&test, "text", false, Boolean).is_none());
    assert!(get_vecstr(&test, "text", df_vec).is_none());
    let test = get_attr!(tbl_good, "SPC-foo", Table::new(), Table).unwrap();
    assert!(get_vecstr(&test, "refs", df_vec).is_none());

    // LOC-tst-core-load-attrs-unit-3:<Test loading valid default types>
    let test = get_attr!(tbl_good, "REQ-foo", Table::new(), Table).unwrap();
    assert!(get_attr!(&test, "disabled", false, Boolean).unwrap() == false);
    assert!(get_attr!(&test, "text", df_str, String).unwrap() == "");
}

#[test]
fn test_check_type() {
    let tbl_good = parse_text(TOML_TEST);
    let df_tbl = Table::new();

    let test = get_attr!(tbl_good, "REQ-bar", df_tbl, Table).unwrap();
    // LOC-tst-core-load-attrs-unit-1:<Test loading valid type>
    fn check_valid(test: &Table) -> LoadResult<Vec<String>> {
        Ok(check_type!(get_vecstr(test, "refs", &Vec::new()), "refs", "name"))
    }
    assert!(check_valid(&test).is_ok());

    let test = get_attr!(tbl_good, "SPC-foo", df_tbl, Table).unwrap();
    fn check_invalid(test: &Table) -> LoadResult<Vec<String>> {
        Ok(check_type!(get_vecstr(test, "refs", &Vec::new()), "refs", "name"))
    }
    assert!(check_invalid(&test).is_err());
}

#[test]
fn test_settings() {
    let tbl_good = parse_text(TOML_TEST);
    let df_tbl = Table::new();
    let mut vars = HashMap::new();

    vars.insert("repo".to_string(), "testrepo".to_string());
    vars.insert("cwd".to_string(), "curdir".to_string());
    let set = Settings::from_table(
        &get_attr!(tbl_good, "settings", df_tbl, Table).unwrap(), &vars).unwrap();
    assert!(set.paths == [PathBuf::from("curdir/test"), PathBuf::from("testrepo/test")]);
    assert!(set.disabled == false);
    let mut expected = HashSet::new();
    expected.insert(".test".to_string());
    assert!(set.repo_names == expected);
}

