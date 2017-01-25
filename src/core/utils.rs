/*  rst: the requirements tracking tool made for developers
 * Copyright (C) 2016  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */

use strfmt;

use dev_prefix::*;
use super::types::*;
use itertools::{Itertools, EitherOrBoth as EoB};

pub fn do_strfmt(s: &str, vars: &HashMap<String, String>, fpath: &PathBuf) -> Result<String> {
    strfmt::strfmt(s, vars).chain_err(|| format!("ERROR at {}: {}", fpath.display(), s.to_string()))
}

pub fn strfmt_ignore_missing<T: fmt::Display>(fmtstr: &str,
                                              vars: &HashMap<String, T>)
                                              -> strfmt::Result<String> {
    let formatter = |mut fmt: strfmt::Formatter| match vars.get(fmt.key) {
        Some(v) => fmt.str(v.to_string().as_str()),
        None => fmt.skip(),
    };
    strfmt::strfmt_map(fmtstr, &formatter)
}

pub fn get_path_str(path: &Path) -> Result<&str> {
    match path.to_str() {
        Some(p) => Ok(p),
        None => Err(ErrorKind::InvalidUnicode(format!("{}", path.display())).into()),
    }
}

/// finds the closest repo dir given a directory
pub fn find_repo(dir: &Path) -> Option<PathBuf> {
    // trace!("start dir: {:?}", dir);
    let dir = env::current_dir().unwrap().join(dir);
    // trace!("abs dir: {:?}", dir);
    assert!(dir.is_dir(), "{}", dir.display());

    let mut dir = dir.as_path();
    fn has_rst_dir(entry: io::Result<fs::DirEntry>) -> bool {
        match entry {
            Err(_) => false,
            Ok(e) => {
                let p = e.path();
                let fname = p.file_name().unwrap().to_str().unwrap();
                // trace!("fname: {:?}", fname);
                fname == ".rst" && p.is_dir()
            }
        }
    }

    loop {
        let mut read_dir = match fs::read_dir(dir) {
            Ok(d) => d,
            Err(_) => return None,
        };
        if read_dir.any(has_rst_dir) {
            return Some(dir.to_path_buf());
        }
        dir = match dir.parent() {
            Some(d) => d,
            None => return None,
        };
        // trace!("dir: {:?}", dir);
    }
}

/// given a path, find the closest dir with the repo identifier
/// and keep track of it
pub fn find_and_insert_repo(dir: &Path, repo_map: &mut HashMap<PathBuf, PathBuf>) -> Result<()> {
    let mut must_insert = false;
    let repo = match repo_map.get(dir) {
        Some(r) => r.to_path_buf(),
        None => {
            let r = match find_repo(dir) {
                Some(r) => r,
                None => {
                    let msg = format!("dir is not part of a repo: {}", dir.display());
                    return Err(ErrorKind::Load(msg).into());
                }
            };
            // can't do this here because of borrowing rules... have to use must_insert
            // repo_map.insert(dir.to_path_buf(), r.to_path_buf());
            must_insert = true;
            r.to_path_buf()
        }
    };
    if must_insert {
        repo_map.insert(dir.to_path_buf(), repo);
    }
    Ok(())
}


/// get the path relative to the `realative_to_dir`
/// for example (foo/bar.txt, bar/baz) => ../../foot/bar.txt
pub fn relative_path(path: &Path, relative_to_dir: &Path) -> PathBuf {
    let mut relative = PathBuf::new();
    let mut remaining = PathBuf::new();
    let mut still_alike = true;
    if path == PARENT_PATH.as_path() {
        return path.to_path_buf();
    }
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
                }
                EoB::Left(a) => remaining.push(a.as_ref()),
                EoB::Right(_) => relative.push(".."),
            }
        }
    }
    relative.extend(remaining.iter());
    relative
}

#[cfg(windows)]
/// windows does terrible things to their path when
/// you get the absolute path -- make it work to be
/// more linux like. We don't need to be accessing
/// other servers or whatever they made this for
///
/// What should be:
///         C:\projects\rst
/// Is instead:
///     \\?\C:\projects\rst
///
/// wut??? I get that they are "speeding up file access"
/// and all... but is this REALLY necessary?
pub fn canonicalize(path: &Path) -> io::Result<PathBuf> {
    let canon = fs::canonicalize(path)?;
    let mut path_iter = canon.iter();
    let prefix = path_iter.next().unwrap();
    let prefix_str = prefix.to_os_string().into_string().unwrap();
    let (icky, new_prefix_str) = prefix_str.split_at(4);
    assert_eq!(icky, r"\\?\");
    let new_prefix = OsString::from(new_prefix_str.to_string());
    let mut new_path = PathBuf::from(&new_prefix);
    new_path.extend(path_iter);

    Ok(new_path)

}

#[cfg(not(windows))]
/// for other systems, just return fs::canonicalize
pub fn canonicalize(path: &Path) -> io::Result<PathBuf> {
    fs::canonicalize(path)
}

#[test]
fn test_relative_path() {
    assert_eq!(relative_path(&PathBuf::from("/foo/bar/txt.t"),
                             &PathBuf::from("/foo/bar/")),
               PathBuf::from("txt.t"));
    assert_eq!(relative_path(&PathBuf::from("/foo/bar/baz/txt.t"),
                             &PathBuf::from("/foo/bar/")),
               PathBuf::from("baz/txt.t"));
    assert_eq!(relative_path(&PathBuf::from("foo/bar/txt.t"), &PathBuf::from("foo/baz/")),
               PathBuf::from("../bar/txt.t"));
    assert_eq!(relative_path(&PathBuf::from("/home/user/projects/what/src/foo/bar.txt"),
                             &PathBuf::from("/home/user/projects/what/reqs/left/right/a/b/c/")),
               PathBuf::from("../../../../../../src/foo/bar.txt"));
}
