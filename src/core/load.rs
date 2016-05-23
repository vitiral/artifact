//! loadrs
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
pub fn get_vecstr(tbl: &Table, attr: &str, default: &Vec<String>)
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
    pub fn from_table(tbl: &Table) -> LoadResult<Settings> {
        let df_vec = Vec::new();
        let str_paths: Vec<String> = check_type!(
            get_vecstr(tbl, "paths", &df_vec), "paths", "settings");
        // let mut paths = vec![];
        // for p in str_paths {
        //     let p = match strfmt(&p, globals) {
        //         Ok(p) => p,
        //         Err(err) => return Err(LoadError::new(err.to_string())),
        //     };
        //     paths.push(PathBuf::from(p));
        // }
        Ok(Settings {
            disabled: check_type!(get_attr!(tbl, "disabled", false, Boolean),
                                  "disabled", "settings"),
            paths: str_paths.iter().map(|s| PathBuf::from(s)).collect(),
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
///     ftable: file-table
///     path: path to this file
///     artifacts: place to put the loaded artifacts
///     settings: place to put the loaded settings
///     variables: place to put the loaded variables
pub fn load_table(ftable: &mut Table, path: &Path,
                  artifacts: &mut Artifacts,
                  settings: &mut Vec<(PathBuf, Settings)>,
                  variables: &mut Vec<(PathBuf, Variables)>)
                  -> LoadResult<u64> {
    let mut msg: Vec<u8> = Vec::new();
    let mut num_loaded: u64 = 0;

    match ftable.remove("settings") {
        Some(Value::Table(t)) => {
            let lset = try!(Settings::from_table(&t));
            if lset.disabled {
                return Ok(0);
            }
            settings.push((path.to_path_buf(), lset));
        }
        None => {},
        _ => return Err(LoadError::new("settings must be a Table".to_string())),
    }

    // TODO: load variables

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

/// Given text load the artifacts
pub fn load_toml(path: &Path, text: &str,
                 artifacts: &mut Artifacts,
                 settings: &mut Vec<(PathBuf, Settings)>,
                 variables: &mut Vec<(PathBuf, Variables)>)
                 -> LoadResult<u64> {
    // parse the text
    let mut parser = Parser::new(text);
    let mut table = match parser.parse() {
        Some(table) => table,
        None => {
            let mut desc = String::new();
            desc.extend(parser.errors.iter().map(|e| e.to_string()));
            return Err(LoadError::new(desc));
        },
    };
    load_table(&mut table, path, artifacts, settings, variables)
}

/// given a file path load the artifacts
///
/// $LOC-core-load-file
pub fn load_file(path: &Path,
                 artifacts: &mut Artifacts,
                 settings: &mut Vec<(PathBuf, Settings)>,
                 variables: &mut Vec<(PathBuf, Variables)>)
                 -> LoadResult<u64> {
    // let mut text: Vec<u8> = Vec::new();

    // read the text
    let mut text = String::new();
    let mut fp = fs::File::open(path).unwrap();
    try!(fp.read_to_string(&mut text).or_else(
        |err| {
            let mut msg = String::new();
            write!(msg, "Error loading path {:?}: {}", path, err);
            Err(LoadError::new(msg))
         }));
    load_toml(path, &text, artifacts, settings, variables)
}

/// LOC-core-load-recursive:<given a path load the raw artifacts from files recursively>
pub fn load_dir(path: &Path,
                loaded_dirs: &mut HashSet<PathBuf>,
                artifacts: &mut Artifacts,
                settings: &mut Vec<(PathBuf, Settings)>,
                variables: &mut Vec<(PathBuf, Variables)>)
                -> LoadResult<u64> {
    // TDOO: if load_path.is_dir()
    let mut num_loaded: u64 = 0;
    let mut error = false;
    // for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
    let mut dirs_to_load: Vec<PathBuf> = Vec::new(); // TODO: references should be possible here...
    let read_dir = match fs::read_dir(path) {
        Ok(d) => d,
        Err(err) => return Err(LoadError::new("E001: ".to_string() + &err.to_string())),
    };
    for entry in read_dir.filter_map(|e| e.ok()) {
        let fpath = entry.path();
        println!("   - possibly loading: {:?}", fpath);
        let ftype = match entry.file_type() {
            Ok(f) => f,
            Err(err) => {
                println!("FAIL while loading from <{}>: {}", fpath.display(), err);
                error = true;
                continue;
            }
        };
        if ftype.is_dir() {
            dirs_to_load.push(fpath.clone());
        } else if ftype.is_file() {
            let ext = match fpath.extension() {
                None => continue,
                Some(ext) => ext,
            };
            if ext != "rsk" {
                continue
            }
            match load_file(fpath.as_path(), artifacts, settings, variables) {
                Ok(n) => {
                    println!("PASS {:<6} loaded from <{}>", n, fpath.display());
                    num_loaded += n;
                }
                Err(err) => {
                    println!("FAIL while loading from <{}>: {}", fpath.display(), err);
                    error = true;
                }
            };
        }
    };
    if num_loaded > 0 {
        for dir in dirs_to_load {
            if loaded_dirs.contains(dir.as_path()) {
                continue;
            }
            loaded_dirs.insert(dir.to_path_buf());
            match load_dir(dir.as_path(), loaded_dirs, artifacts, settings, variables) {
                Ok(n) => num_loaded += n,
                Err(_) => error = true,
            }
        }
    }
    if error {
        return Err(LoadError::new("ERROR: some files failed to load".to_string()));
    } else {
        Ok(num_loaded)
    }
}

pub fn find_repo<'a>(dir: &'a Path, repo_names: &HashSet<String>) -> Option<&'a Path> {
    let mut dir = dir;
    assert!(dir.is_dir());
    loop {
        let mut read_dir = match fs::read_dir(dir) {
            Ok(d) => d,
            Err(_) => return None,
        };
        if read_dir.any(|e|
            match e {
                Err(_) => false,
                Ok(e) => {
                    let p = e.path();
                    let fname = p.file_name().unwrap().to_str().unwrap();
                    repo_names.contains(fname) && p.is_dir()
                }
            }) {
            return Some(dir);
        }
        dir = match dir.parent() {
            Some(d) => d,
            None => return None,
        };
    }
}


fn get_path_str<'a>(path: &'a Path) -> LoadResult<&'a str> {
    match path.to_str() {
        Some(p) => Ok(p),
        None => Err(LoadError::new(
            "detected invalid unicode in path name: ".to_string() +
            path.to_string_lossy().as_ref())),
    }
}

fn resolve_settings(settings: &mut Settings,
                    repo_map: &mut HashMap<PathBuf, PathBuf>,
                    loaded_settings: &Vec<(PathBuf, Settings)>)
                    -> LoadResult<()> {
    // first pull out all of the repo_names
    for ps in loaded_settings.iter() {
        let ref s: &Settings = &ps.1;
        for rn in &s.repo_names {
            settings.repo_names.insert(rn.clone());
        }
    }

    // now resolve all path names
    let mut vars: HashMap<String, String> = HashMap::new();
    for ps in loaded_settings.iter() {
        let ref settings_item: &Settings = &ps.1;

        // load the default global variables {cwd} and {repo}
        let fpath = ps.0.clone();
        let dir = fpath.parent().unwrap();
        let dir_str = try!(get_path_str(dir));

        // TODO: for full windows compatibility you will probably want to support OsStr
        // here... I just don't want to
        vars.insert("cwd".to_string(), dir_str.to_string());
        let mut must_insert = false;
        let repo = match repo_map.get(dir) {
            Some(r) => r.to_path_buf(),
            None => {
                let r = match find_repo(dir, &settings.repo_names) {
                    Some(r) => r,
                    None => return Err(LoadError::new("dir is not part of a repo: ".to_string() +
                                                      dir_str)),
                };
                // can't do this here because of borrowing rules... have to use must_insert
                // repo_map.insert(dir.to_path_buf(), r.to_path_buf());
                must_insert = true;
                r.to_path_buf()
            }
        };
        if must_insert {
            repo_map.insert(dir.to_path_buf(), repo.clone());
        }

        vars.insert("repo".to_string(), try!(get_path_str(repo.as_path())).to_string());

        // push resolved paths
        for p in settings_item.paths.iter() {
            let p = match strfmt(p.to_str().unwrap(), &vars) {
                Ok(p) => p,
                Err(e) => {
                    let mut msg = String::new();
                    write!(msg, "ERROR at {}: {}", fpath.to_string_lossy().as_ref(), e.to_string());
                    return Err(LoadError::new(msg));
                }
            };
            settings.paths.push_back(PathBuf::from(p));
        }
    }

    Ok(())
}



/// given a valid path, load all paths
pub fn load_path(path: &Path) -> LoadResult<(Artifacts, Settings, Variables,
                                             HashMap<PathBuf, PathBuf>)>{
    let mut artifacts = Artifacts::new();
    let mut settings = Settings::new();
    let mut variables = Variables::new();
    let mut loaded_dirs: HashSet<PathBuf> = HashSet::new();
    let mut loaded_settings: Vec<(PathBuf, Settings)> = Vec::new();
    let mut loaded_variables: Vec<(PathBuf, Variables)> = Vec::new();
    let mut repo_map: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut num_loaded: u64 = 0;

    if path.is_file() {
        num_loaded += try!(load_file(path, &mut artifacts, &mut loaded_settings,
                                     &mut loaded_variables));
        try!(resolve_settings(&mut settings, &mut repo_map, &loaded_settings));
    } else if path.is_dir() {
        settings.paths.push_back(path.to_path_buf());
    } else {
        return Err(LoadError::new("File is not valid type: ".to_string() +
                                  path.to_string_lossy().as_ref()));
    }

    println!(" - Started loading: {:?}", path);

    // LOC-core-load-parts-1:<load and validate all paths recursively>
    while settings.paths.len() > 0 {
        loaded_settings.clear();
        loaded_variables.clear();
        let dir = settings.paths.pop_front().unwrap(); // it has len, it better pop!

        println!(" - Loading: {:?}", dir);
        // load the files
        if loaded_dirs.contains(&dir) {
            continue
        }
        loaded_dirs.insert(dir.to_path_buf());
        num_loaded += match load_dir(dir.as_path(), &mut loaded_dirs,
                                     &mut artifacts, &mut loaded_settings,
                                     &mut loaded_variables) {
            Ok(n) => n,
            Err(err) => {
                let mut msg = String::new();
                write!(msg, "Error loading <{}>: {}", dir.to_string_lossy().as_ref(), err);
                return Err(LoadError::new(msg));
            }
        };
        // resolve the project-level settings (paths, repo_names, etc)
        try!(resolve_settings(&mut settings, &mut repo_map, &loaded_settings));
    }

    // TODO: LOC-core-load-parts-2:<load and validate global variables>
    // LOC-core-load-parts-3:<resolve variables in text fields>
    // LOC-core-load-parts-4:<auto-creation of missing prefix artifacts>
    // LOC-core-load-parts-5:<linking of artifacts>

    Ok((artifacts, settings, variables, repo_map))
}
