//! vars module
//! used by the load module to resolve and apply loaded variables
//! also contains settings resolution because it is similar

use std::fs;
use std::env;
use std::clone::Clone;
use std::path::{Path, PathBuf};
use std::convert::AsRef;
use std::collections::{HashMap, HashSet, VecDeque};
use std::iter::FromIterator;

// Traits
use std::io::{Write, Read};
use std::fmt::Write as WriteStr;

// crates
use strfmt;

use core::types::*;

lazy_static!{
    pub static ref DEFAULT_GLOBALS: HashSet<String> = HashSet::from_iter(
        ["repo", "cwd"].iter().map(|s| s.to_string()));
    pub static ref SPC: VecDeque<char> = VecDeque::from_iter(vec!['#', 'S', 'P', 'C', '-']);
    pub static ref TST: VecDeque<char> = VecDeque::from_iter(vec!['#', 'T', 'S', 'T', '-']);
}

/// finds the closest repo dir given a directory
pub fn find_repo(dir: &Path, repo_names: &HashSet<String>) -> Option<PathBuf> {
    // trace!("start dir: {:?}", dir);
    let dir = env::current_dir().unwrap().join(dir);
    // trace!("abs dir: {:?}", dir);
    assert!(dir.is_dir());

    let mut dir = dir.as_path();

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
                    // trace!("fname: {:?}", fname);
                    repo_names.contains(fname) && p.is_dir()
                }
            }) {
            return Some(dir.to_path_buf());
        }
        dir = match dir.parent() {
            Some(d) => d,
            None => return None,
        };
        // trace!("dir: {:?}", dir);
    }
}

fn do_strfmt(s: &str, vars: &HashMap<String, String>, fpath: &PathBuf)
             -> LoadResult<String> {
    match strfmt::strfmt(s, &vars) {
        Ok(s) => Ok(s),
        Err(e) => {
            let mut msg = String::new();
            write!(msg, "ERROR at {}: {}", fpath.to_string_lossy().as_ref(), e.to_string())
                .unwrap();
            return Err(LoadError::new(msg));
        }
    }
}

/// #SPC-core-load-settings-resolve:<resolve all informaiton related to settings>
pub fn resolve_settings(settings: &mut Settings,
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

        let fpath = ps.0.clone();
        let cwd = fpath.parent().unwrap();
        let cwd_str = try!(get_path_str(cwd));

        // TODO: for full windows compatibility you will probably want to support OsStr
        // here... I just don't want to
        // [#SPC-core-settings-vars]
        vars.insert("cwd".to_string(), cwd_str.to_string());
        try!(find_and_insert_repo(cwd, repo_map, &settings.repo_names));
        let repo = repo_map.get(cwd).unwrap();
        vars.insert("repo".to_string(), try!(get_path_str(repo.as_path())).to_string());

        // push resolved paths
        for p in settings_item.paths.iter() {
            let p = try!(do_strfmt(p.to_str().unwrap(), &vars, &fpath));
            settings.paths.push_back(PathBuf::from(p));
        }

        // TODO: it is possible to be able to use all global variables in code_paths
        // push resolved code_paths
        for p in settings_item.code_paths.iter() {
            let p = try!(do_strfmt(p.to_str().unwrap(), &vars, &fpath));
            settings.code_paths.push_back(PathBuf::from(p));
        }

        // push resolved exclude_code_paths
        for p in settings_item.exclude_code_paths.iter() {
            let p = try!(do_strfmt(p.to_str().unwrap(), &vars, &fpath));
            settings.exclude_code_paths.push_back(PathBuf::from(p));
        }
    }
    Ok(())
}

/// LOC-find-repo:<given a path, find the closest dir with the repo identifier
///     and keep track of it>
pub fn find_and_insert_repo(dir: &Path, repo_map: &mut HashMap<PathBuf, PathBuf>,
                        repo_names: &HashSet<String>)
                        -> LoadResult<()> {
    let mut must_insert = false;
    let repo = match repo_map.get(dir) {
        Some(r) => r.to_path_buf(),
        None => {
            let r = match find_repo(&dir, repo_names) {
                Some(r) => r,
                None => {
                    let mut msg = String::new();
                    write!(msg, "dir is not part of a repo: {}", dir.to_string_lossy().as_ref())
                        .unwrap();
                    return Err(LoadError::new(msg));
                }
            };
            // can't do this here because of borrowing rules... have to use must_insert
            // repo_map.insert(dir.to_path_buf(), r.to_path_buf());
            must_insert = true;
            r.to_path_buf()
        },
    };
    if must_insert {
        repo_map.insert(dir.to_path_buf(), repo);
    }
    Ok(())
}


/// resolves default vars from a file (cwd and repo)
/// and inserts into variables
/// #SPC-core-vars-resolve-default
pub fn resolve_default_vars(vars: &Variables, fpath: &Path,
                            variables: &mut Variables,
                            repo_map: &mut HashMap<PathBuf, PathBuf>,
                            repo_names: &HashSet<String>)
                            -> LoadResult<()> {
    let cwd = fpath.parent().unwrap();
    let mut fmtvars = Variables::new();
    fmtvars.insert("cwd".to_string(), cwd.to_str().unwrap().to_string());
    try!(find_and_insert_repo(cwd, repo_map, repo_names));
    fmtvars.insert("repo".to_string(), repo_map.get(cwd).unwrap()
                     .to_str().unwrap().to_string());
    let mut error = false;
    for (k, v) in vars {
        // format only the cwd and repo variables
        let var = match strfmt::strfmt_options(v.as_str(), &fmtvars, true) {
            Ok(v) => v,
            Err(e) => {
                // [#SPC-core-load-error-vars-1]
                error!("error formatting: {}", e.to_string());
                error = true;
                continue;
            }
        };
        match variables.insert(k.clone(), var) {
            Some(_) => {
                // [#SPC-core-load-error-vars-2]
                error!("global var {:?} exists twice, one at {:?}", k, fpath);
                error = true;
            }
            None => {}
        }
    }
    if error {
        // [#SPC-core-load-error-vars-return-1]
        return Err(LoadError::new("errors while resolving default variables".to_string()));
    }
    Ok(())
}

/// continues to resolve variables until all are resolved
/// - done if no vars were resolved in a pass and no errors
/// - error if no vars were resolved in a pass and there were errors
/// #SPC-core-vars-resolve-user
pub fn resolve_vars(variables: &mut Variables) -> LoadResult<()> {
    // keep resolving variables until all are resolved
    let mut msg = String::new();
    let mut keys: Vec<String> = variables.keys().map(|s| s.clone()).collect();
    let mut errors = Vec::new();
    let mut num_changed;
    let mut remove_keys = DEFAULT_GLOBALS.clone();
    loop {
        keys = keys.iter().filter(|k| !remove_keys.contains(k.as_str()))
            .map(|s| s.clone()).collect();
        num_changed = 0;
        errors.clear();
        remove_keys.clear();
        for k in &keys {
            let var = variables.remove(k.as_str()).unwrap();
            match strfmt::strfmt(var.as_str(), &variables) {
                Ok(s) => {
                    // TODO: being able to know whether changes were made would remove need
                    // to compare input to output
                    if var != s {
                        // var was changed, but it still might have {} in it
                        num_changed += 1;
                    } else {
                        // no errors, but also didn't change. It is done evaluating
                        remove_keys.insert(k.clone());
                    }
                    variables.insert(k.clone(), s);
                }
                Err(e) => match e {
                    strfmt::FmtError::Invalid(e) => return Err(LoadError::new(e.to_string())),
                    strfmt::FmtError::KeyError(_) => {
                        // [#SPC-core-load-error-vars-3]
                        errors.push(k.clone());
                        // reinsert original value
                        variables.insert(k.clone(), var);
                    }
                }
            }
        }
        if num_changed == 0 {  // no items changed, we are either done or failed
            if errors.len() == 0 {
                break;
            } else {
                // unresolved errors
                // [#SPC-core-load-error-vars-return-2]
                keys = keys.iter().filter(|k| !remove_keys.contains(k.as_str()))
                    .map(|s| s.clone()).collect();
                write!(msg, "Could not resolve some globals: {:?}\ngot related errors: {:?}",
                       keys, errors).unwrap();
                return Err(LoadError::new(msg));
            }
        }
    }
    Ok(())
}

/// use the variables to fill in the text fields of all artifacts
/// LOC-artifacts-vars
pub fn fill_text_fields(artifacts: &mut Artifacts,
                        settings: &Settings,
                        variables: &mut Variables,
                        repo_map: &mut HashMap<PathBuf, PathBuf>)
                        -> LoadResult<()> {
    // resolve all text blocks in artifacts
    let mut error = false;
    let mut errors: Vec<(&str, strfmt::FmtError)> = Vec::new();
    for (name, art) in artifacts.iter_mut() {
        trace!("filling in {}", name);
        errors.clear();
        let cwd = art.path.parent().expect("no-path-parent").to_path_buf();
        try!(find_and_insert_repo(&cwd, repo_map, &settings.repo_names));
        variables.insert("cwd".to_string(), cwd.to_str().expect("utf-path").to_string());
        variables.insert("repo".to_string(), repo_map.get(&cwd).expect("repo_map")
                            .to_str().expect("utf-path").to_string());

        // evaluate text
        match strfmt::strfmt(art.text.as_str(), &variables) {
            Ok(t) => art.text = t,
            Err(err) => errors.push(("text field", err)),
        };
        let mut refs = Vec::new();
        for r in &art.refs {
            match strfmt::strfmt(r.as_str(), &variables) {
                Ok(r) => refs.push(r),
                // [#SPC-core-load-error-text-1]
                Err(err) => errors.push(("ref", err)),
            }
        }
        art.refs = refs;
        if errors.len() > 0 {
            // [#SPC-core-load-error-text-3]
            error!(" resolving variables on [{:?}] {} failed: {:?}", art.path, name, errors);
            error = true;
        }
    }

    if error {
        // [#SPC-core-load-error-text-return]
        return Err(LoadError::new("failure to resolve artifact text fields".to_string()));
    }
    trace!("Done filling");
    Ok(())
}

fn get_path_str<'a>(path: &'a Path) -> LoadResult<&'a str> {
    match path.to_str() {
        Some(p) => Ok(p),
        None => Err(LoadError::new(
            "detected invalid unicode in path name: ".to_string() +
            path.to_string_lossy().as_ref())),
    }
}

pub fn find_locs_text(path: &Path,
                      text: &str,
                      locs: &mut HashMap<ArtName, Loc>)
                      -> bool {
    let mut error = false;
    let text = text;
    let mut prev: VecDeque<char> = VecDeque::with_capacity(5);
    let mut prev_char = ' ';
    let mut start_pos = 0;
    let mut start_col = 0;
    let (mut pos, mut line, mut col) = (0, 1, 0); // line starts at 1
    // pretty simple parse tree... just do it ourselves!
    // Looking for #LOC-[a-z0-9_-] case insensitive
    for c in text.chars() {
        if prev == *SPC || prev == *TST {  // TODO: I'm sure this is not as fast as possible
            if prev_char == ' ' {
                start_pos = pos - 5;
                start_col = col - 5;
            }
            match c {
                'a'...'z' | 'A'...'Z' | '0'...'9' | '-' | '_' => {
                    prev_char = c;  // still reading a valid artifact name
                }
                _ => {  // valid #ART is finished
                    if prev_char != ' ' { // "SPC- ", etc is actually invalid
                        let art_start = start_pos + 1; // + 1 because of '#'
                        let (_, end) = text.split_at(art_start);
                        // if last char is '-' ignore it
                        let (name, _) = match prev_char {
                            '-' => end.split_at(pos - art_start - 1),
                            _ => end.split_at(pos - art_start),
                        };
                        let locname = ArtName::from_str(name).unwrap();
                        debug!("Found loc: {}", locname);
                        let loc = Loc {
                            path: path.to_path_buf(),
                            line_col: (line, start_col)
                        };
                        match locs.insert(locname, loc) {
                            None => {},
                            Some(l) => {
                                error!("detected overlapping loc {} in files: {:?} and {}",
                                        name, l, path.display());
                                error = true;
                            }
                        }
                        prev_char = ' ';
                    }
                    prev.pop_front();
                    prev.push_back(c);
                },
            }
        } else {
            if prev.len() == 5 {
                prev.pop_front();
            }
            prev.push_back(c);
        }
        match c {
            '\n' => {
                line += 1;
                col = 0;
            }
            _ => col += 1,
        };
        pos += 1;
    }
    error
}

/// [#SPC-core-load-loc-text]
/// given text, the path to the text, and the locations to add onto
/// extract all the locations from the text and return whether there
/// was an error
pub fn find_locs_file(path: &Path,
                      locs: &mut HashMap<ArtName, Loc>)
                      -> bool {
    debug!("resolving locs at: {:?}", path);
    let mut text = String::new();
    match fs::File::open(path) {
        Ok(mut f) => match f.read_to_string(&mut text) {
            Ok(_) => {},
            Err(e) => {
                error!("while reading from <{}>: {}", path.display(), e);
                return true;
            }
        },
        Err(e) => {
            error!("while loading from <{}>: {}", path.display(), e);
            return true;
        },
    }
    find_locs_text(path, &text, locs)
}

/// recursively find all locs given a directory
fn find_locs_dir(path: &PathBuf, loaded_dirs: &mut HashSet<PathBuf>,
                 locs: &mut HashMap<ArtName, Loc>)
                 -> bool {
    loaded_dirs.insert(path.to_path_buf());
    let read_dir = match fs::read_dir(path) {
        Ok(d) => d,
        Err(err) => {
            error!("while loading from dir <{}>: {}", path.display(), err);
            return true;
        }
    };
    let mut error = false;
    let mut dirs_to_load: Vec<PathBuf> = Vec::new(); // TODO: use references
    for entry in read_dir.filter_map(|e| e.ok()) {
        let fpath = entry.path();
        // don't parse .rsk files for locations
        match fpath.extension() {
            None => {},
            Some(ext) => if ext == "rsk" {
                continue
            }
        }
        let ftype = match entry.file_type() {
            Ok(f) => f,
            Err(err) => {
                error!("while loading from <{}>: {}", fpath.display(), err);
                error = true;
                continue;
            }
        };
        if ftype.is_dir() {
            dirs_to_load.push(fpath.clone());
        } else if ftype.is_file() {
            match find_locs_file(&fpath, locs) {
                true => error = true,
                false => {},
            }
        }
    };

    for d in dirs_to_load {
        if loaded_dirs.contains(&d) {
            continue;
        }
        match find_locs_dir(&d, loaded_dirs, locs) {
            true => error = true,
            false => {},
        }
    }
    error
}

/// search through the code_paths in settings to find all valid locs
pub fn find_locs(settings: &mut Settings) -> LoadResult<HashMap<ArtName, Loc>> {
    info!("parsing code files for artifacts...");
    let mut locs: HashMap<ArtName, Loc> = HashMap::new();
    let mut loaded_dirs: HashSet<PathBuf> = HashSet::from_iter(
        settings.exclude_code_paths.iter().map(|p| p.to_path_buf()));
    // first make sure the excluded directories exist
    for d in loaded_dirs.iter() {
        if !d.exists() {
            let mut msg = String::new();
            write!(msg, "excluded path {} does not exist!", d.display()).unwrap();
            return Err(LoadError::new(msg));
        }
    }
    debug!("initial excluded code paths: {:?}", loaded_dirs);
    let mut error = false;
    while settings.code_paths.len() > 0 {
        let dir = settings.code_paths.pop_front().unwrap(); // it has len, it better pop!
        if loaded_dirs.contains(&dir) {
            continue
        }
        debug!("Loading from code: {:?}", dir);
        match find_locs_dir(&dir, &mut loaded_dirs, &mut locs) {
            false => {},
            true => return Err(LoadError::new("encountered errors while finding locations".to_string())),
        }
    }
    if error {
        Err(LoadError::new("encountered errors when loading locations from code".to_string()))
    } else {
        Ok(locs)
    }
}

/// [#SPC-core-load-loc-resolve]
pub fn attach_locs(artifacts: &mut Artifacts, locs: &HashMap<ArtName, Loc>) {
    for (lname, loc) in locs {
        let artifact = match artifacts.get_mut(&lname) {
            Some(a) => a,
            None => continue,
        };
        artifact.loc = Some(loc.clone());
    }
}
