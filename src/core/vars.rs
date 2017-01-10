/*  rst: the requirements tracking tool made for developers
    Copyright (C) 2016  Garrett Berg <@vitiral, vitiral@gmail.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the Lesser GNU General Public License as published 
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the Lesser GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/
//! vars module
//! used by the load module to resolve and apply loaded variables
//! also contains settings resolution because it is similar

use dev_prefix::*;
use super::types::*;
use super::utils;

use strfmt;

lazy_static!{
    pub static ref DEFAULT_GLOBALS: HashSet<String> = HashSet::from_iter(
        ["repo", "cwd"].iter().map(|s| s.to_string()));
}

/// resolves default vars from a file (cwd and repo)
/// and inserts into variables
pub fn resolve_default_vars(vars: &Variables, fpath: &Path,
                            variables: &mut Variables,
                            repo_map: &mut HashMap<PathBuf, PathBuf>)
                            -> Result<()> {
    let cwd = fpath.parent().unwrap();
    let mut fmtvars = Variables::new();
    fmtvars.insert("cwd".to_string(), cwd.to_str().unwrap().to_string());
    try!(utils::find_and_insert_repo(cwd, repo_map));
    fmtvars.insert("repo".to_string(), repo_map.get(cwd).unwrap()
                     .to_str().unwrap().to_string());
    let mut error = false;
    for (k, v) in vars {
        // format only the cwd and repo variables
        let var = match utils::strfmt_ignore_missing(v.as_str(), &fmtvars) {
            Ok(v) => v,
            Err(e) => {
                error!("error formatting: {}", e.to_string());
                error = true;
                continue;
            }
        };
        if variables.insert(k.clone(), var).is_some() {
            error!("global var {:?} exists twice, one at {:?}", k, fpath);
            error = true;
        }
    }
    if error {
        return Err(ErrorKind::InvalidVariable(
            "errors while resolving default variables".to_string()).into());
    }
    Ok(())
}

/// continues to resolve variables until all are resolved
/// - done if no vars were resolved in a pass and no errors
/// - error if no vars were resolved in a pass and there were errors
pub fn resolve_vars(variables: &mut Variables) -> Result<()> {
    // keep resolving variables until all are resolved
    let mut msg = String::new();
    let mut keys: Vec<String> = variables.keys().cloned().collect();
    let mut errors = Vec::new();
    let mut num_changed;
    let mut remove_keys = DEFAULT_GLOBALS.clone();
    loop {
        keys = keys.iter().filter(|k| !remove_keys.contains(k.as_str()))
            .cloned().collect();
        num_changed = 0;
        errors.clear();
        remove_keys.clear();
        for k in &keys {
            let var = variables.remove(k.as_str()).unwrap();
            match strfmt::strfmt(var.as_str(), variables) {
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
                    strfmt::FmtError::Invalid(_) | strfmt::FmtError::TypeError(_) =>
                        return Err(ErrorKind::StrFmt(e).into()),
                    strfmt::FmtError::KeyError(_) => {
                        errors.push(k.clone());
                        // reinsert original value
                        variables.insert(k.clone(), var);
                    },
                }
            }
        }
        if num_changed == 0 {  // no items changed, we are either done or failed
            if errors.is_empty() {
                break;
            } else {
                // unresolved errors
                keys = keys.iter().filter(|k| !remove_keys.contains(k.as_str()))
                    .cloned().collect();
                let msg = format!(
                    "Could not resolve some globals: {:?}\ngot related errors: {:?}",
                   keys, errors);
                return Err(ErrorKind::InvalidVariable(msg).into());
            }
        }
    }
    Ok(())
}

/// use the variables to fill in the text fields of all artifacts
pub fn fill_text_fields(artifacts: &mut Artifacts,
                        variables: &mut Variables,
                        repo_map: &mut HashMap<PathBuf, PathBuf>)
                        -> Result<()> {
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
        match strfmt::strfmt(art.text.value.as_str(), variables) {
            Ok(t) => art.text.value = t,
            Err(err) => errors.push(("text field", err)),
        };
        if !errors.is_empty() {
            error!(" resolving variables on [{:?}] {} failed: {:?}", art.path, name, errors);
            error = true;
        }
    }

    if error {
        return Err(ErrorKind::InvalidTextVariables.into());
    }
    trace!("Done filling");
    Ok(())
}

/// resolve raw loaded variables, replacing default and user-defined globals
/// recursively
/// partof: #SPC-vars
pub fn resolve_loaded_vars(variables_map: &HashMap<PathBuf, Variables>,
                           repo_map: &mut HashMap<PathBuf, PathBuf>)
                           -> Result<Variables> {
    let mut variables = Variables::new();
    debug!("Resolving default globals in variables, see SPC-vars.1");
    for pv in variables_map.iter() {
        let p = pv.0;
        let v = pv.1;
        try!(resolve_default_vars(&v, p.as_path(), &mut variables, repo_map));
    }
    debug!("Resolving variables, see SPC-vars.2");
    try!(resolve_vars(&mut variables));
    Ok(variables)
}
