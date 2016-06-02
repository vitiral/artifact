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

/// LOC-load-settings-resolve:<resolve all informaiton related to settings>
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

        // load the default global variables {cwd} and {repo}
        let fpath = ps.0.clone();
        let cwd = fpath.parent().unwrap();
        let cwd_str = try!(get_path_str(cwd));

        // TODO: for full windows compatibility you will probably want to support OsStr
        // here... I just don't want to
        // LOC-core-settings-vars
        vars.insert("cwd".to_string(), cwd_str.to_string());
        try!(find_and_insert_repo(cwd, repo_map, &settings.repo_names));
        let repo = repo_map.get(cwd).unwrap();
        vars.insert("repo".to_string(), try!(get_path_str(repo.as_path())).to_string());

        // push resolved paths
        for p in settings_item.paths.iter() {
            let p = match strfmt::strfmt(p.to_str().unwrap(), &vars) {
                Ok(p) => p,
                Err(e) => {
                    let mut msg = String::new();
                    write!(msg, "ERROR at {}: {}", fpath.to_string_lossy().as_ref(), e.to_string())
                        .unwrap();
                    return Err(LoadError::new(msg));
                }
            };
            settings.paths.push_back(PathBuf::from(p));
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
/// LOC-core-vars-resolve-default
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
                error!("error formatting: {}", e.to_string());
                error = true;
                continue;
            }
        };
        match variables.insert(k.clone(), var) {
            Some(_) => {
                error!("global var {:?} exists twice, one at {:?}", k, fpath);
                error = true;
            }
            None => {}
        }
    }
    if error {
        return Err(LoadError::new("errors while resolving default variables".to_string()));
    }
    Ok(())
}

/// continues to resolve variables until all are resolved
/// - done if no vars were resolved in a pass and no errors
/// - error if no vars were resolved in a pass and there were errors
/// LOC-core-vars-resolve-user
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
        let cwd = art.path.parent().unwrap().to_path_buf();
        try!(find_and_insert_repo(&cwd, repo_map, &settings.repo_names));
        variables.insert("cwd".to_string(), cwd.to_str().unwrap().to_string());
        variables.insert("repo".to_string(), repo_map.get(&cwd).unwrap()
                            .to_str().unwrap().to_string());

        // evaluate text
        match strfmt::strfmt(art.text.as_str(), &variables) {
            Ok(t) => art.text = t,
            Err(err) => errors.push(("text field", err)),
        };
        let mut refs = Vec::new();
        for r in &art.refs {
            match strfmt::strfmt(r.as_str(), &variables) {
                Ok(r) => refs.push(r),
                Err(err) => errors.push(("ref", err)),
            }
        }
        art.refs = refs;
        let mut set_loc = art.loc.clone();
        if let Some(ref loc) = art.loc {
            match strfmt::strfmt(loc.path.to_str().unwrap(), &variables) {
                Ok(l) => {
                    trace!("loc path set to: {}", l);
                    trace!("using variables: {:?}", variables);
                    set_loc = Some(Loc {
                        path: PathBuf::from(l.as_str()),
                        line_col: None,
                    });
                }
                Err(err) => errors.push(("loc", err)),
            }
        }
        art.loc = set_loc;
        if errors.len() > 0 {
            error!(" resolving variables on [{:?}] {} failed: {:?}", art.path, name, errors);
            error = true;
        }
    }

    if error {
        return Err(LoadError::new("failure to resolve artifact text fields".to_string()));
    }
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

pub fn resolve_locs(artifacts: &mut Artifacts) -> LoadResult<()> {
    info!("resolving locations...");
    let mut paths: HashSet<PathBuf> = HashSet::new();
    let mut looking_for: HashSet<ArtName> = HashSet::new();
    // get all valid paths and the locations we are looking for
    for (name, artifact) in artifacts.iter() {
        if let Some(ref l) = artifact.loc {
            looking_for.insert(name.clone());
            if !paths.contains(l.path.as_path()) {
                paths.insert(l.path.clone());
            }
        }
    };
    paths.remove(Path::new(""));

    // analyze all files for valid locations
    let mut error = false;
    // values are                   (path, line,  col)
    let mut locs: HashMap<ArtName, (PathBuf, usize, usize)> = HashMap::new();
    for path in paths {
        let mut fd = match fs::File::open(path.clone()) {
            Ok(fd) => fd,
            Err(_) => continue,
        };
        let mut s = String::new();
        fd.read_to_string(&mut s).unwrap();

        let spc: VecDeque<char> = VecDeque::from_iter(vec!['S', 'P', 'C', '-']);
        let tst: VecDeque<char> = VecDeque::from_iter(vec!['T', 'S', 'T', '-']);
        let mut prev: VecDeque<char> = VecDeque::with_capacity(4);
        let mut prev_char = ' ';
        let mut start_pos = 0;
        let mut start_col = 0;
        let mut loc_part = ' ';  // ' ' represents blank
        let (mut pos, mut line, mut col) = (0, 1, 0); // line starts at 1
        // pretty simple parse tree... just do it ourselves!
        // Looking for LOC-[a-z0-9_-] case insensitive
        for c in s.chars() {
            if prev == spc || prev == tst {
                if prev_char == ' ' {
                    start_pos = pos;
                    start_col = col;
                }
                match c {
                    'a'...'z' | 'A'...'Z' | '0'...'9' | '-' | '_' => {
                        prev_char = c;  // still reading a valid artifact name
                    }
                    _ => {  // valid LOC is finished
                        if prev_char != ' ' { // "SPC- ", etc is actually invalid
                            let (_, end) = s.split_at(start_pos);
                            // if last char is '-' ignore it
                            let (name, _) = match prev_char {
                                '-' => end.split_at(pos - start_pos - 1),
                                _ => end.split_at(pos - start_pos),
                            };
                            let locname = ArtName::from_str(name).unwrap();
                            if looking_for.contains(&locname) {
                                // only do checking if the loc actually exists
                                // check for overlap on insert
                                match locs.insert(locname,
                                                (path.clone(), line, start_col)) {
                                    None => {},
                                    Some(l) => {
                                        error!("detected overlapping loc {} in files: {:?} and {:?}",
                                            name, l.0, path.as_path());
                                        error = true;
                                    }
                                }
                            } else {
                                warn!("Found loc that is not a member of an artifact: {}", name);
                            }
                            prev_char = ' ';
                        }
                        prev.pop_back();
                        prev.push_front(c);
                    },
                }
            } else {
                if prev.len() == 4 {
                    prev.pop_back();
                }
                prev.push_front(c);
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
    }
    if error {
        return Err(LoadError::new("Overlapping keys found in src loc".to_string()));
    }
    debug!("Found file locs: {:?}", locs);

    // now fill in the location values
    for (lname, info) in locs {
        let artifact = artifacts.get_mut(&lname).unwrap();
        let (path, line, col) = info;
        let aloc = artifact.loc.iter_mut().next().unwrap();
        if aloc.path != path {
            error!("found {} at path {:?}, but {} has it set at {:?}",
                lname, path, lname, aloc.path);
            error = true;
            continue;
        };
        aloc.line_col = Some((line, col));
    }

    if error {
        return Err(LoadError::new("Invalid paths".to_string()));
    }
    Ok(())
}
