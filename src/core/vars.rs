//! vars module
//! used by the load module to resolve and apply loaded variables
//! also contains settings resolution because it is similar

use std::fs;
use std::env;
use std::clone::Clone;
use std::path::{Path, PathBuf};
use std::convert::AsRef;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

// Traits
use std::io::{Write};
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

/// continues to resolve variables until all are resolved
/// - done if no vars were resolved in a pass and no errors
/// - error if no vars were resolved in a pass and there were errors
/// LOC-core-vars-resolve
pub fn resolve_vars(variables: &mut Variables,
                    var_paths: &HashMap<String, PathBuf>,
                    repo_map: &mut HashMap<PathBuf, PathBuf>,
                    repo_names: &HashSet<String>,
                    )
                    -> LoadResult<()> {
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
            let int_cwd = var_paths.get(k).unwrap();
            let cwd = int_cwd.parent().unwrap();
            // let cwd = var_paths.get(k).unwrap().parent().unwrap();
            variables.insert("cwd".to_string(), cwd.to_str().unwrap().to_string());
            try!(find_and_insert_repo(&cwd, repo_map, repo_names));
            variables.insert("repo".to_string(), repo_map.get(cwd).unwrap()
                             .to_str().unwrap().to_string());
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
                Err(_) => {
                    // TODO: strfmt should give two types of error,
                    // FmtError or NameError
                    // we should fail immediately on fmterror
                    // with a beter error msg
                    errors.push(k.clone());
                    // reinsert original value
                    variables.insert(k.clone(), var);
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
                        loc: loc.loc.clone(),
                        path: PathBuf::from(l.as_str()),
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
