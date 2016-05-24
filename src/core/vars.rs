//! vars module
//! used by the load module to resolve variables

use std::fs;
use std::clone::Clone;
use std::path::{Path, PathBuf};
use std::convert::AsRef;
use std::collections::{HashMap, HashSet};

// Traits
use std::io::{Write};
use std::fmt::Write as WriteStr;

// crates
use strfmt;

use core::types::*;

/// finds the closest repo dir given a directory
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
                repo_map: &HashMap<PathBuf, PathBuf>)
                -> LoadResult<()> {
    // keep resolving variables until all are resolved
    let mut msg = String::new();
    let mut keys: Vec<String> = variables.keys().map(|s| s.clone()).collect();
    let mut errors = Vec::new();
    let mut num_changed;
    let mut remove_keys = HashSet::new();
    loop {
        keys = keys.iter().filter(|k| !remove_keys.contains(k.as_str()))
            .map(|s| s.clone()).collect();
        num_changed = 0;
        errors.clear();
        remove_keys.clear();
        for k in &keys {
            let var = variables.remove(k.as_str()).unwrap();
            let cwd = var_paths.get(k).unwrap().parent().unwrap();
            variables.insert("cwd".to_string(), cwd.to_str().unwrap().to_string());
            variables.insert("repo".to_string(), repo_map.get(cwd).unwrap()
                             .to_str().unwrap().to_string());
            match strfmt::strfmt(var.as_str(), &variables) {
                Ok(s) => {
                    if var != s {
                        num_changed += 1;
                    }
                    remove_keys.insert(k.clone());
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
                write!(msg, "Could not resolve some globals: {:?}", errors).unwrap();
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
                Ok(l) => set_loc = Some(Loc {
                    loc: loc.loc.clone(),
                    path: PathBuf::from(l.as_str()),
                }),
                Err(err) => errors.push(("loc", err)),
            }
        }
        art.loc = set_loc;
        if errors.len() > 0 {
            println!("ERROR: resolving variables on {} failed: {:?}", name, errors);
            error = true;
        }
    }

    if error {
        return Err(LoadError::new("failure to resolve artifact text fields".to_string()));
    }
    Ok(())
}
