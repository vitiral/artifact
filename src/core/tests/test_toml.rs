

/// This is mainly to test some assumptions about TOML
use super::*;

use toml::{Parser, Value, Table};

#[test]
fn test_assumptions() {
    // This test is to prove that is impossible to write artifact names
    // such as ART-foo.1
    // since this is made impossible by the toml format itself,
    // we can assume that such artifacts don't exist when we create them automatically
    // by parsing the text.
    let toml = "\
    [test]
    key = 'value'

    [test.1]
    key = 'value.1'
    ";

    let tbl = parse_text(toml);
    let test = get_table(&tbl, "test");
    let value = match test.get("key").unwrap() {
        &Value::String(ref s) => s,
        _ => unreachable!(),
    };
    assert_eq!(value, "value");
    let test_1 = get_table(&test, "1");
    let value = match test_1.get("key").unwrap() {
        &Value::String(ref s) => s,
        _ => unreachable!(),
    };
    assert_eq!(value, "value.1");
}
