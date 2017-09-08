use std::sync::atomic::{AtomicUsize, Ordering as AtomicOrdering};
use std::io;

use itertools::{EitherOrBoth as EoB, Itertools};
use uuid::Uuid;


use dev_prefix::*;
use types::*;

lazy_static!{
    static ref INCREMENTING_ID: AtomicUsize = AtomicUsize::new(1);
    pub static ref UUID: Uuid = Uuid::new_v4();
}

/// used for artifact ids
pub fn unique_id() -> u64 {
    INCREMENTING_ID.fetch_add(1, AtomicOrdering::SeqCst) as u64
}


/// get the path relative to the `realative_to_dir`
/// for example (foo/bar.txt, bar/baz) => ../../foo/bar.txt
pub fn relative_path(path: &Path, relative_to_dir: &Path) -> PathBuf {
    let mut relative = PathBuf::new();
    let mut remaining = PathBuf::new();
    let mut still_alike = true;
    for zipped in path.components().zip_longest(relative_to_dir.components()) {
        if still_alike {
            still_alike = match zipped {
                EoB::Both(a, b) => a == b, // consume idential part of path
                EoB::Left(_) => false,     // relative_to_dir is root of path
                _ => unreachable!("Paths have no identical root"),
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

#[test]
fn test_relative_path() {
    assert_eq!(
        relative_path(
            &PathBuf::from("/foo/bar/txt.t"),
            &PathBuf::from("/foo/bar/"),
        ),
        PathBuf::from("txt.t")
    );
    assert_eq!(
        relative_path(
            &PathBuf::from("/foo/bar/baz/txt.t"),
            &PathBuf::from("/foo/bar/"),
        ),
        PathBuf::from("baz/txt.t")
    );
    assert_eq!(
        relative_path(&PathBuf::from("foo/bar/txt.t"), &PathBuf::from("foo/baz/")),
        PathBuf::from("../bar/txt.t")
    );
    assert_eq!(
        relative_path(
            &PathBuf::from("/home/user/projects/what/src/foo/bar.txt"),
            &PathBuf::from("/home/user/projects/what/reqs/left/right/a/b/c/"),
        ),
        PathBuf::from("../../../../../../src/foo/bar.txt")
    );
}


/// finds the closest repo given a directory
pub fn find_repo(dir: &Path) -> Result<PathBuf> {
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
                fname == ".art" && p.is_dir()
            }
        }
    }

    loop {
        let mut read_dir = fs::read_dir(dir)?;
        if read_dir.any(has_rst_dir) {
            let repo = canonicalize(dir)?;
            return Ok(repo);
        }
        dir = match dir.parent() {
            Some(d) => d,
            None => {
                return Err(
                    io::Error::new(io::ErrorKind::NotFound, "repo not found").into(),
                )
            }
        };
        // trace!("dir: {:?}", dir);
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use test_data;
    #[test]
    fn test_find_repo() {
        let simple = &test_data::TSIMPLE_DIR;
        assert_eq!(find_repo(simple.as_path()).unwrap(), simple.as_path());
        assert_eq!(
            find_repo(simple.join("design").join("lvl_1").as_path()).unwrap(),
            simple.as_path()
        );
        assert!(find_repo(env::temp_dir().as_path()).is_err());
    }
}

#[cfg(windows)]
/// windows does terrible things to their path when
/// you get the absolute path -- make it work to be
/// more linux like. We don't need to be accessing
/// other servers or whatever they made this for
///
/// What should be:
///         C:\projects\artifact
/// Is instead:
///     \\?\C:\projects\artifact
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
/// for other systems, just return `fs::canonicalize`
pub fn canonicalize(path: &Path) -> io::Result<PathBuf> {
    fs::canonicalize(path)
}
