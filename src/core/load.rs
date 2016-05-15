//! load.rs
//! loading of raw artifacts from files and text

use std::ascii::AsciiExt;
use std::fs;
use std::clone::Clone;
use std::path::Path;
use std::convert::AsRef;
use std::io::{Read, Write};
use std::collections::HashMap;

use regex::Regex;
use walkdir::WalkDir;
use toml::{Parser, Value, Table};

use core::types::{Artifact, Artifacts, ArtTypes, LoadError, LoadResult};

lazy_static!{
    // must start with artifact type, followed by "-", followed by at least 1 valid character
    // cannot end with "-"
    pub static ref ART_VALID: Regex = Regex::new(
        r"(REQ|SPC|RSK|TST|LOC)-[A-Z0-9_-]*[A-Z0-9_]\z").unwrap();
}

/// LOC-name-check:<check that name is valid>
fn artifact_name_valid(name: &str) -> bool {
    let check = name.to_ascii_uppercase();
    ART_VALID.is_match(&check)
}

fn fix_artifact_name(name: &str) -> String {
    name.replace(" ", "")
}

#[test]
/// LOC-tst-name-check: <check that name combinations raise correct errors>
fn test_name() {
    // valid names
    for name in vec!["REQ-foo", "REQ-foo-2", "REQ-foo2", "REQ-foo2", "REQ-foo-bar-2_3",
                     "SPC-foo", "RSK-foo", "TST-foo", "LOC-foo"] {
        assert!(artifact_name_valid(name));
    }

    for name in vec!["REQ-foo*", "REQ-foo\n", "REQ-foo-"] {
        assert!(!artifact_name_valid(name))
    }

    // spaces
    assert!(fix_artifact_name("   R E Q    -    f   o  o") == "REQ-foo");
}

// macro_rules! get_attr {
//     ($tbl: expr, $attr: expr, $default: expr, $defaults: expr,
//      $ty: ident) => {
//         match $tbl.get($attr) {
//             // If the value is in the table, return the value
//             Some(&Value::$ty(ref v)) => Some(v.clone()),
//             // otherwise, get from the defaults
//             None => {
//                 match $defaults {
//                     None => Some($default),  // there are no defaults
//                     Some(ref d) => {
//                         match d.get($attr) {
//                             Some(&Value::$ty(ref v)) => Some(v.clone()),  // got from defaults
//                             None => Some($default),
//                             _ => None, // invalid type
//                         }
//                     },
//                 }
//             },
//             _ => None, // invalid type
//         }
//     }
// }

// /// LOC-core-load-table-check:<check the type to make sure it matches>
// macro_rules! check_type {
//     ($value: expr, $attr: expr, $name: expr) => {
//         match $value {
//             Some(v) => v,
//             None => {
//                 let mut msg = Vec::new();
//                 write!(&mut msg, "{} has invalid attribute: {}", $name, $attr).unwrap();
//                 return Err(LoadError::new(String::from_utf8(msg).unwrap()));
//             }
//         }
//     }
// }

// /// LOC-core-load-table:<load a table from toml>
// pub fn load_table(artifacts: &mut Artifacts, ftable: &mut Table, path: &Path) -> LoadResult<u64> {
//     // let &mut artifacs = Vec::new();
//     let invalid_type = |name: &str, attr: &str| -> LoadError{
//         let mut msg = Vec::new();
//         write!(&mut msg, "{} has invalid attribute: {}", name, attr).unwrap();
//         LoadError::new(String::from_utf8(msg).unwrap())
//     };

//     let defaults = match ftable.remove("defaults") {
//         Some(Value::Table(t)) => Some(t),
//         None => None,
//         _ => return Err(invalid_type(&"defaults".to_string(), "defaults")),
//     };
//     let mut msg: Vec<u8> = Vec::new();
//     let mut num_loaded: u64 = 0;
//     for (name, value) in ftable.iter() {
//         // check that the artifact's name is valid
//         if !artifact_name_valid(&name) {
//             write!(&mut msg, "invalid name: {}", name).unwrap();
//             return Err(LoadError::new(String::from_utf8(msg).unwrap()));
//         }
//         // get the artifact table
//         let art_tbl: &Table = match value {
//             &Value::Table(ref t) => t,
//             _ => {
//                 write!(&mut msg, "All top-level values must be a table: {}", name).unwrap();
//                 return Err(LoadError::new(String::from_utf8(msg).unwrap()));
//             }
//         };
//         // check for overlap
//         if artifacts.contains_key(name) {
//             write!(&mut msg, "Overlapping key found <{}> other key at: {}",
//                    name, artifacts.get(name).unwrap().path.display()).unwrap();
//             return Err(LoadError::new(String::from_utf8(msg).unwrap()));
//         }
//         // check if artifact is active
//         if !check_type!(get_attr!(
//                 art_tbl, "active", true, defaults, Boolean), 
//                              "active", name) {
//             continue
//         }
//         let artifact_type = {
//             if      name.starts_with("REQ") {ArtTypes::REQ}
//             else if name.starts_with("SPC") {ArtTypes::SPC}
//             else if name.starts_with("RSK") {ArtTypes::RSK}
//             else if name.starts_with("TST") {ArtTypes::TST}
//             else {unreachable!()}
//         };

//         artifacts.insert(name.clone(), Artifact{
//             // loaded vars
//             ty: artifact_type,
//             name: name.clone(),
//             path: path.to_path_buf(),
//             text: check_type!(get_attr!(art_tbl, "text", "".to_string(), defaults, String),
//                               "text", name),
//             extra: check_type!(get_attr!(art_tbl, "extra", "".to_string(), defaults, String),
//                                "exta", name),
//             partof_str: check_type!(get_attr!(art_tbl, "partof", "".to_string(), defaults, String),
//                                     "partof", name),
//             loc_str: check_type!(get_attr!(art_tbl, "loc", "".to_string(), defaults, String),
//                                  "loc", name),
//             done: check_type!(get_attr!(art_tbl, "done", false, defaults, Boolean),
//                               "done", name),
//             ignore: check_type!(get_attr!(art_tbl, "ignore", false, defaults, Boolean),
//                                 "ignore", name),

//             // calculated vars
//             partof: Vec::new(),
//             parts: Vec::new(),
//             loc: None,
//         });
//         num_loaded += 1;
//     }
//     return Ok(num_loaded);
// }

// /// Given text load the artifacts
// pub fn load_text(artifacts: &mut Artifacts, text: &str, path: &Path) -> LoadResult<u64> {
//     // parse the text
//     let mut parser = Parser::new(text);
//     let mut table = match parser.parse() {
//         Some(table) => table,
//         None => {
//             let mut desc = String::new();
//             desc.extend(parser.errors.iter().map(|e| e.to_string()));
//             return Err(LoadError::new(desc));
//         },
//     };
//     load_table(artifacts, &mut table, path)
// }

// /// given a file path load the artifacts
// ///
// /// $LOC-core-load-file
// pub fn load_file(artifacts: &mut Artifacts, load_path: &Path) -> LoadResult<u64> {
//     // let mut text: Vec<u8> = Vec::new();

//     // read the text
//     let mut text = String::new();
//     let mut fp = fs::File::open(load_path).unwrap();
//     try!(fp.read_to_string(&mut text).or_else(
//         |err| Err(LoadError::new(err.to_string()))));
//     load_text(artifacts, &text, load_path)
// }

// /// LOC-core-load-recursive:<given a path load the raw artifacts from files recursively>
// pub fn recursive_raw_load<P: AsRef<Path>>(load_path: P) -> LoadResult<Artifacts> {
//     // TDOO: if load_path.is_dir()
//     let mut error = false;
//     let mut artifacts: HashMap<String, Artifact> = HashMap::new();
//     for entry in WalkDir::new(&load_path).into_iter().filter_map(|e| e.ok()) {
//         let ftype = entry.file_type();
//         if ftype.is_dir() {
//             continue
//         }
//         let path = entry.path();
//         let ext = match path.extension() {
//             None => continue,
//             Some(ext) => ext,
//         };
//         if ext != "toml" {
//             continue
//         }
//         match load_file(&mut artifacts, path) {
//             Ok(n) => println!("PASS {:<6} loaded from <{}>", n, path.display()),
//             Err(err) => {
//                 println!("FAIL while loading from <{}>: {}", path.display(), err);
//                 error = true;
//             }
//         };
//     };
//     if error {
//         return Err(LoadError::new("ERROR: some files failed to load".to_string()));
//     }
//     else {
//         Ok(artifacts)
//     }
// }

#[cfg(test)]
static TOML_GOOD: &'static str = "
[defaults]
done = true
text = 'foo'
[REQ-foo]
[SPC-foo]
[RSK-foo]
[TST-foo]
[REQ-bar]
text = 'bar'
done = false
";

static TOML_BAD: &'static str = "[REQ-bad]\ndone = '100%'";
static TOML_OVERLAP: &'static str = "[REQ-foo]\n";


#[cfg(test)]
fn parse_text(t: &str) -> Table {
    Parser::new(t).parse().unwrap()
}

#[cfg(test)]
fn get_table<'a>(tbl: &'a Table, attr: &str) -> &'a Table {
    match tbl.get(attr).unwrap() {
        &Value::Table(ref t) => t,
        _ => unreachable!()
    }
}

// #[test]
// fn test_get_attr() {
//     let tbl_good = parse_text(TOML_GOOD);
//     let defaults = Some(get_table(&tbl_good, "defaults"));
//     let empty = Some(Table::new());

//     let test = get_attr!(tbl_good, "REQ-bar", Table::new(), defaults, Table).unwrap();
//     // LOC-tst-core-load-unit-1:<Test loading valid existing types>
//     assert!(get_attr!(&test, "done", false, defaults, Boolean).unwrap() == false);
//     assert!(get_attr!(&test, "done", false, empty, Boolean).unwrap() == false);
//     assert!(get_attr!(&test, "done", true, defaults, Boolean).unwrap() == false);
//     assert!(get_attr!(&test, "done", true, empty, Boolean).unwrap() == false);
//     assert!(get_attr!(&test, "text", "".to_string(), defaults, String).unwrap() == "bar");
//     assert!(get_attr!(&test, "text", "".to_string(), empty, String).unwrap() == "bar");

//     // LOC-tst-core-load-unit-2:<Test loading invalid existing types>
//     assert!(get_attr!(&test, "done", "".to_string(), defaults, String).is_none());
//     assert!(get_attr!(&test, "text", false, empty, Boolean).is_none());

//     // LOC-tst-core-load-unit-3:<Test loading valid default types>
//     let test = get_attr!(tbl_good, "REQ-foo", Table::new(), defaults, Table).unwrap();
//     assert!(get_attr!(&test, "done", false, empty, Boolean).unwrap() == false);
//     assert!(get_attr!(&test, "done", true, empty, Boolean).unwrap());
//     assert!(get_attr!(&test, "done", false, defaults, Boolean).unwrap());
//     assert!(get_attr!(&test, "text", "".to_string(), empty, String).unwrap() == "");
//     assert!(get_attr!(&test, "text", "".to_string(), defaults, String).unwrap() == "foo");

//     // LOC-tst-core-load-unit-4:<Test loading invalid default types>
//     assert!(get_attr!(&test, "done", "".to_string(), defaults, String).is_none());
//     assert!(get_attr!(&test, "text", false, defaults, Boolean).is_none());
// }

// #[test]
// /// LOC-tst-core-load-text-1:<Test loading raw text>
// fn test_load_text() {
//     let path = Path::new("");
//     let mut artifacts: HashMap<String, Artifact> = HashMap::new();
//     load_text(&mut artifacts, TOML_GOOD, &path).unwrap();
//     assert!(load_text(&mut artifacts, TOML_OVERLAP, &path).is_err());
//     assert!(load_text(&mut artifacts, TOML_BAD, &path).is_err());
// }
