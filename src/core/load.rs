//! load.rs
//! loading of raw artifacts from files and text

use std::ascii::AsciiExt;
use std::fs;
use std::clone::Clone;
use std::path::{Path, PathBuf};
use std::convert::AsRef;
use std::collections::{HashMap, HashSet};

// Traits
use std::io::{Read, Write};
use std::fmt::Write as WriteStr;
use std::iter::FromIterator;

use walkdir::WalkDir;
use toml::{Parser, Value, Table};
use strfmt::strfmt;

use core::types::*;

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
    // remove spaces
    assert!(fix_artifact_name("   R E Q    -    f   o  o   ") == "REQ-foo");
}

macro_rules! get_attr {
    ($tbl: expr, $attr: expr, $default: expr, $ty: ident) => {
        match $tbl.get($attr) {
            // If the value is in the table, return the value
            Some(&Value::$ty(ref v)) => Some(v.clone()),
            // otherwise return the default
            None => Some($default.clone()),
            // If it's the wrong type, return None (Err)
            _ => None,
        }
    }
}

/// only one type is in an array, so make this custom
fn get_vecstr(tbl: &Table, attr: &str, default: &Vec<String>)
              -> Option<Vec<String>> {
    match tbl.get(attr) {
        // if the value is in the table, try to get it's elements
        Some(&Value::Array(ref a)) => {
            let mut out: Vec<String> = Vec::with_capacity(a.len());
            for v in a {
                match v {
                    &Value::String(ref s) => out.push(s.clone()),
                    _ => return None,  // error: invalid type
                }
            }
            Some(out)
        }
        None => Some(default.clone()), // value doesn't exist, return default
        _ => None,  // error: invalid type
    }
}

/// LOC-core-load-table-check:<check the type to make sure it matches>
macro_rules! check_type {
    ($value: expr, $attr: expr, $name: expr) => {
        match $value {
            Some(v) => v,
            None => {
                let mut msg = Vec::new();
                write!(&mut msg, "{} has invalid attribute: {}", $name, $attr).unwrap();
                return Err(LoadError::new(String::from_utf8(msg).unwrap()));
            }
        }
    }
}

impl Settings {
    fn from_table(tbl: &Table, globals: &Variables) -> LoadResult<Settings> {
        let df_vec = Vec::new();
        let str_paths: Vec<String> = check_type!(
            get_vecstr(tbl, "paths", &df_vec), "paths", "settings");
        let mut paths = vec![];

        for p in str_paths {
            let p = match strfmt(&p, globals) {
                Ok(p) => p,
                Err(err) => return Err(LoadError::new(err.to_string())),
            };
            paths.push(PathBuf::from(p));
        }
        Ok(Settings {
            disabled: check_type!(get_attr!(tbl, "disabled", false, Boolean),
                                  "disabled", "settings"),
            paths: paths,
            repo_names: HashSet::from_iter(check_type!(
                get_vecstr(tbl, "repo_names", &df_vec), "repo_names", "settings")),
        })
    }
}

fn _parse_partof<I>(raw: &mut I, in_brackets: bool) -> LoadResult<Vec<String>>
    where I: Iterator<Item = char>
{
    // hello-[there, you-[are, great]]
    // hello-there, hello-you-are, hello-you-great
    let mut strout = String::new();
    let mut current = String::new();
    loop {
        let c = match raw.next() {
            Some(c) => c,
            None => {
                if in_brackets {
                    return Err(LoadError::new("brackets are not closed".to_string()));
                }
                break;
            }
        };
        println!("{:?}", c);
        match c {
            ' ' => {}, // ignore whitespace
            '[' => {
                if current == "" {
                    return Err(LoadError::new("cannot have '[' after characters ',' or ']' \
                                               or at start of string".to_string()));
                }
                for p in try!(_parse_partof(raw, true)) {
                    strout.write_str(&current).unwrap();
                    strout.write_str(&p).unwrap();
                    strout.push(',');
                }
                current.clear();
            }
            ']' => break,
            ',' => {
                strout.write_str(&current).unwrap();
                strout.push(',');
                current.clear();
            }
            _ => current.push(c),
        }
    }
    strout.write_str(&current).unwrap();
    Ok(strout.split(",").filter(|s| s != &"").map(|s| s.to_string()).collect())
}

fn parse_partof(partof_str: &str) -> LoadResult<HashSet<ArtName>> {
    let strs = try!(_parse_partof(&mut partof_str.chars(), false));
    let mut out = HashSet::new();
    for s in strs {
        let n = try!(ArtName::from_str(s.as_str()));
        out.insert(n);
    }
    Ok(out)
}

#[test]
fn test_parse_partof() {
    assert_eq!(_parse_partof(&mut "hi, ho".chars(), false).unwrap(), ["hi", "ho"]);
    assert_eq!(_parse_partof(&mut "hi-[ho, he]".chars(), false).unwrap(), ["hi-ho", "hi-he"]);
    assert_eq!(_parse_partof(
        &mut "hi-[ho, he], he-[ho, hi, ha-[ha, he]]".chars(), false).unwrap(),
        ["hi-ho", "hi-he", "he-ho", "he-hi", "he-ha-ha", "he-ha-he"]);
    assert!(_parse_partof(&mut "[]".chars(), false).is_err());
    assert!(_parse_partof(&mut "[hi]".chars(), false).is_err());
    assert!(_parse_partof(&mut "hi-[ho, [he]]".chars(), false).is_err());
    assert!(_parse_partof(&mut "hi-[ho, he".chars(), false).is_err());
}

impl Artifact {
    fn from_table(name: &ArtName, path: &Path, tbl: &Table) -> LoadResult<Artifact> {
        let df_str = "".to_string();
        let df_vec: Vec<String> = vec![];

        let partof_str = check_type!(get_attr!(tbl, "partof", df_str, String),
                                    "partof", name);
        let loc_str = check_type!(get_attr!(tbl, "loc", df_str, String),
                                 "loc", name);
        let loc = match loc_str.as_str() {
            "" => None,
            _ => Some(try!(Loc::from_str(loc_str.as_str()))),
        };

        Ok(Artifact{
            // loaded vars
            ty: name.get_type(),
            path: path.to_path_buf(),
            text: check_type!(get_attr!(tbl, "text", df_str, String),
                              "text", name),
            refs: check_type!(get_vecstr(tbl, "refs", &df_vec), "refs", name),
            partof: try!(parse_partof(&partof_str)),
            loc: loc,

            // calculated vars
            parts: HashSet::new(),
            completed: None,
            tested: None,
        })
    }
}

/// LOC-core-load-table:<load a table from toml>
/// inputs:
///     artifacts: place to put the loaded artifacts
///     settings: place to put the loaded settings
///     variables: place to put the loaded variables
///     ftable: file-table
///     default_globals: default global variables
pub fn load_table(artifacts: &mut Artifacts, settings: &mut Settings,
                  ftable: &mut Table, path: &Path,
                  default_globals: &Variables)
                  -> LoadResult<u64> {
    let mut msg: Vec<u8> = Vec::new();
    let mut num_loaded: u64 = 0;

    match ftable.remove("settings") {
        Some(Value::Table(t)) => {
            let lset = try!(Settings::from_table(&t, default_globals));
            if lset.disabled {
                return Ok(0);
            }
            for p in lset.paths {
                if settings.paths.contains(&p) {
                    return Err(LoadError::new(
                        "Cannot have a path listed twice".to_string() + &p.to_string_lossy()));
                }
                if !p.exists() {
                    return Err(LoadError::new(
                        "path in settings['path'] does not exist: ".to_string() +
                            &p.to_string_lossy()));
                }
                settings.paths.push(p.clone());
            }
            settings.repo_names.extend(lset.repo_names);
        }
        None => {},
        _ => return Err(LoadError::new("settings must be a Table".to_string())),
    }

    for (name, value) in ftable.iter() {
        let aname = try!(ArtName::from_str(name));
        // get the artifact table
        let art_tbl: &Table = match value {
            &Value::Table(ref t) => t,
            _ => {
                write!(&mut msg, "All top-level values must be a table: {}", name).unwrap();
                return Err(LoadError::new(String::from_utf8(msg).unwrap()));
            }
        };
        // check for overlap
        if let Some(overlap) = artifacts.get(&aname) {
            write!(&mut msg, "Overlapping key found <{}> other key at: {}",
                name, overlap.path.display()).unwrap();
            return Err(LoadError::new(String::from_utf8(msg).unwrap()));
        }
        // check if artifact is active
        if check_type!(get_attr!(art_tbl, "disabled", false, Boolean),
                       "disabled", name) {
            continue
        }
        let artifact = try!(Artifact::from_table(&aname, path, art_tbl));
        artifacts.insert(aname, artifact);
        num_loaded += 1;
    }
    return Ok(num_loaded);
}

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

// ##################################################
// functions for tests

#[cfg(test)]

#[cfg(test)]

#[test]
/// LOC-tst-core-artifacts-types:<test loading and checking of enum types>
fn test_types() {

}

// #[test]
// /// LOC-tst-core-load-text-1:<Test loading raw text>
// fn test_load_text() {
//     let path = Path::new("");
//     let mut artifacts: HashMap<String, Artifact> = HashMap::new();
//     load_text(&mut artifacts, TOML_GOOD, &path).unwrap();
//     assert!(load_text(&mut artifacts, TOML_OVERLAP, &path).is_err());
//     assert!(load_text(&mut artifacts, TOML_BAD, &path).is_err());
// }
