//! vars module
//! used by the load module to resolve and apply loaded variables
//! also contains settings resolution because it is similar

use std::env;
use super::types::*;
use super::utils;

lazy_static!{
    pub static ref DEFAULT_GLOBALS: HashSet<String> = HashSet::from_iter(
        ["repo", "cwd"].iter().map(|s| s.to_string()));
}

/// resolves default vars from a file (cwd and repo)
/// and inserts into variables
/// #SPC-core-vars-resolve-default
pub fn resolve_default_vars(vars: &Variables, fpath: &Path,
                            variables: &mut Variables,
                            repo_map: &mut HashMap<PathBuf, PathBuf>)
                            -> LoadResult<()> {
    let cwd = fpath.parent().unwrap();
    let mut fmtvars = Variables::new();
    fmtvars.insert("cwd".to_string(), cwd.to_str().unwrap().to_string());
    try!(utils::find_and_insert_repo(cwd, repo_map));
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
        try!(utils::find_and_insert_repo(&cwd, repo_map));
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

/// resolve raw loaded variables, replacing default and user-defined globals
/// recursively
/// partof: #SPC-vars
pub fn resolve_loaded_vars(mut loaded_vars: Vec<(PathBuf, Variables)>,
                           repo_map: &mut HashMap<PathBuf, PathBuf>)
                           -> LoadResult<Variables> {
    let mut variables = Variables::new();
    debug!("Resolving default globals in variables, see SPC-vars.1");
    for pv in loaded_vars.drain(0..) {
        let p = pv.0;
        let v = pv.1;
        try!(resolve_default_vars(&v, p.as_path(), &mut variables, repo_map));
    }
    debug!("Resolving variables, see SPC-vars.2");
    try!(resolve_vars(&mut variables));
    Ok(variables)
}
