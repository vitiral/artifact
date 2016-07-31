use std::env;
use super::types::*;

pub fn do_strfmt(s: &str, vars: &HashMap<String, String>, fpath: &PathBuf)
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

pub fn get_path_str<'a>(path: &'a Path) -> LoadResult<&'a str> {
    match path.to_str() {
        Some(p) => Ok(p),
        None => Err(LoadError::new(
            "detected invalid unicode in path name: ".to_string() +
                path.to_string_lossy().as_ref())),
    }
}

/// finds the closest repo dir given a directory
pub fn find_repo(dir: &Path) -> Option<PathBuf> {
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
                                fname == ".rsk" && p.is_dir()
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

/// LOC-find-repo:<given a path, find the closest dir with the repo identifier
///     and keep track of it>
pub fn find_and_insert_repo(dir: &Path, repo_map: &mut HashMap<PathBuf, PathBuf>)
                        -> LoadResult<()> {
    let mut must_insert = false;
    let repo = match repo_map.get(dir) {
        Some(r) => r.to_path_buf(),
        None => {
            let r = match find_repo(&dir) {
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

