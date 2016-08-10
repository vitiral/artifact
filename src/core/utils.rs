use std::env;
use std::fmt;

use strfmt;

use super::types::*;
use itertools::{Itertools, EitherOrBoth as EoB};

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

pub fn strfmt_ignore_missing<T: fmt::Display>(fmtstr: &str, vars: &HashMap<String, T>)
                                              -> strfmt::Result<String>{
    let formatter = |mut fmt: strfmt::Formatter| {
        match vars.get(fmt.key) {
            Some(v) => fmt.str(v.to_string().as_str()),
            None => fmt.skip(),
        }
    };
    strfmt::strfmt_map(fmtstr, &formatter)
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
                                fname == ".rst" && p.is_dir()
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


/// get the path relative to the realative_to_dir
/// for example (foo/bar.txt, bar/baz) => ../../foot/bar.txt
pub fn relative_path(path: &Path, relative_to_dir: &Path) -> PathBuf {
    let mut relative = PathBuf::new();
    let mut remaining = PathBuf::new();
    let mut still_alike = true;
    for zipped in path.components().zip_longest(relative_to_dir.components()) {
        if still_alike {
            still_alike = match zipped {
                EoB::Both(a, b) => a == b,  // consume idential part of path
                EoB::Left(_) => false,  // relative_to_dir is root of path
                _ => unreachable!("paths have no identical root"),
            }
        }
        if !still_alike {
            match zipped {
                EoB::Both(a, _) => {
                    relative.push("..");
                    remaining.push(a.as_ref());
                },
                EoB::Left(a) => remaining.push(a.as_ref()),
                EoB::Right(_) => relative.push(".."),
            }
        }
    }
    relative.extend(remaining.iter());
    relative
}

#[test]
fn test_relative_path() {
    assert_eq!(relative_path(&PathBuf::from("/foo/bar/txt.t"),
                             &PathBuf::from("/foo/bar/")),
               PathBuf::from("txt.t"));
    assert_eq!(relative_path(&PathBuf::from("/foo/bar/baz/txt.t"),
                             &PathBuf::from("/foo/bar/")),
               PathBuf::from("baz/txt.t"));
    assert_eq!(relative_path(&PathBuf::from("foo/bar/txt.t"),
                             &PathBuf::from("foo/baz/")),
               PathBuf::from("../bar/txt.t"));
    assert_eq!(relative_path(&PathBuf::from("/home/user/projects/what/src/foo/bar.txt"),
                             &PathBuf::from("/home/user/projects/what/reqs/left/right/a/b/c/")),
               PathBuf::from("../../../../../../src/foo/bar.txt"));
}
