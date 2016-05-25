//! loadrs
//! loading of raw artifacts from files and text

use std::fs;
use std::clone::Clone;
use std::path::{Path, PathBuf};
use std::convert::AsRef;
use std::collections::{HashMap, HashSet, VecDeque};

// Traits
use std::io::{Read, Write};
use std::fmt::Write as WriteStr;
use std::iter::FromIterator;

// crates
use toml::{Parser, Value, Table};
use time;

// modules
use core::types::*;
use core::vars::{resolve_vars, resolve_settings, fill_text_fields};

lazy_static!{
    pub static ref DEFAULT_GLOBALS: HashSet<String> = HashSet::from_iter(
        ["repo", "globals"].iter().map(|s| s.to_string()));
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
    /// LOC-core-settings-from_table:<load a settings object from a table>
    pub fn from_table(tbl: &Table) -> LoadResult<Settings> {
        let df_vec = Vec::new();
        let str_paths: Vec<String> = check_type!(
            get_vecstr(tbl, "paths", &df_vec), "paths", "settings");
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
            completed: -1.0,
            tested: -1.0,
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

    match ftable.remove("globals") {
        Some(Value::Table(t)) => {
            let mut lvars = Variables::new();
            for (k, v) in t {
                if DEFAULT_GLOBALS.contains(k.as_str()) {
                    return Err(LoadError::new("cannot use variables: repo, cwd".to_string()));
                }
                lvars.insert(k.clone(), match v {
                    Value::String(s) => s.to_string(),
                    _ => return Err(LoadError::new(
                        k.to_string() + " global var must be of type str")),
                });
            }
            variables.push((path.to_path_buf(), lvars));
        }
        None => {},
        _ => return Err(LoadError::new("globals must be a Table".to_string())),
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
            let mut msg = String::new();
            for e in &parser.errors {
                let (line, col) = parser.to_linecol(e.lo);
                write!(msg, "[{}:{}] {}, ", line, col, e.desc).unwrap();
            }
            // write!(msg, "Could not parse []: {}", parser.errors);
            // desc.extend(parser.errors.iter().map(|e| e.to_string()));
            return Err(LoadError::new(msg));
        },
    };
    load_table(&mut table, path, artifacts, settings, variables)
}

/// given a file path load the artifacts
///
/// LOC-core-load-file
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
            write!(msg, "Error loading path {:?}: {}", path, err).unwrap();
            Err(LoadError::new(msg))
         }));
    load_toml(path, &text, artifacts, settings, variables)
}

/// LOC-core-load-dir:<given a path load the raw artifacts from files recursively>
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
    // process all the files in the directory. Process directories later
    for entry in read_dir.filter_map(|e| e.ok()) {
        let fpath = entry.path();
        let ftype = match entry.file_type() {
            Ok(f) => f,
            Err(err) => {
                error!("while loading from <{}>: {}", fpath.display(), err);
                error = true;
                continue;
            }
        };
        if ftype.is_dir() {
            dirs_to_load.push(fpath.clone()); // load directories after files have been loaded
        } else if ftype.is_file() {
            let ext = match fpath.extension() {
                None => continue,
                Some(ext) => ext,
            };
            if ext != "rsk" { // only load rsk files
                continue
            }
            match load_file(fpath.as_path(), artifacts, settings, variables) {
                Ok(n) => num_loaded += n,
                Err(err) => {
                    error!("while loading from <{}>: {}", fpath.display(), err);
                    error = true;
                }
            };
        }
    };
    if num_loaded > 0 { // REQ-core-load-recursive: don't recurse if no .rsk files are in dir
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

fn default_repo_names() -> HashSet<String> {
    let mut repo_names: HashSet<String> = HashSet::new();
    repo_names.insert(".git".to_string());
    repo_names.insert(".hg".to_string());
    repo_names.insert(".svn".to_string());
    repo_names
}

/// given a valid path, load all paths
/// linking does not occur in this step
/// LOC-core-load-path
pub fn load_path(path: &Path) -> LoadResult<(Artifacts, Settings)>{
    let mut artifacts = Artifacts::new();
    let mut settings = Settings{disabled: false, paths:VecDeque::new(),
                                repo_names: default_repo_names()};
    let mut variables = Variables::new();
    let mut loaded_dirs: HashSet<PathBuf> = HashSet::new();
    let mut loaded_settings: Vec<(PathBuf, Settings)> = Vec::new();
    let mut loaded_variables: Vec<(PathBuf, Variables)> = Vec::new();
    let mut repo_map: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut num_loaded: u64 = 0;
    let mut msg = String::new();

    let start = time::get_time();
    info!("Loading artifact files:");
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

    // LOC-core-load-parts-1:<load and validate all paths recursively>
    while settings.paths.len() > 0 {
        loaded_settings.clear();
        let dir = settings.paths.pop_front().unwrap(); // it has len, it better pop!

        debug!("Loading: {:?}", dir);
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
                write!(msg, "Error loading <{}>: {}", dir.to_string_lossy().as_ref(), err).unwrap();
                return Err(LoadError::new(msg));
            }
        };
        // resolve the project-level settings after each directory is recursively loaded
        try!(resolve_settings(&mut settings, &mut repo_map, &loaded_settings));
    }

    info!("Organizing variables...");

    let mut error = false;
    let mut var_paths: HashMap<String, PathBuf> = HashMap::new();
    for pv in loaded_variables.drain(0..) {
        let p = pv.0;
        let vars = pv.1;
        for (k, v) in vars {
            match variables.insert(k.clone(), v) {
                Some(_) => {
                    error!("global var {:?} exists twice, one at {:?}", k, p);
                    error = true;
                }
                None => {}
            }
            var_paths.insert(k, p.clone());
        }
    }
    if error {
        return Err(LoadError::new("Error while organizing variables".to_string()));
    }

    info!("Resolving variables...");
    try!(resolve_vars(&mut variables, &var_paths, &mut repo_map, &settings.repo_names));

    info!("Filling in variables for text fields...");
    try!(fill_text_fields(&mut artifacts, &settings, &mut variables, &mut repo_map));

    let total = time::get_time() - start;
    info!("Done loading: {} artifacts loaded successfullly in {:.3} seconds",
          num_loaded, total.num_milliseconds() as f64 * 1e-3);

    Ok((artifacts, settings))
}
