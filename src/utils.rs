use itertools::{Itertools, EitherOrBoth as EoB};
use toml::{Parser, Table};

use dev_prefix::*;
use types::*;

/// just parse toml using a std error for this library
pub fn parse_toml(toml: &str) -> Result<Table> {
    let mut parser = Parser::new(toml);
    match parser.parse() {
        Some(table) => Ok(table),
        None => {
            let mut locs = String::new();
            for e in &parser.errors {
                let (line, col) = parser.to_linecol(e.lo);
                write!(locs, "[{}:{}] {}, ", line, col, e.desc).unwrap();
            }
            Err(ErrorKind::TomlParse(locs).into())
        }
    }
}


/// get the path relative to the `realative_to_dir`
/// for example (foo/bar.txt, bar/baz) => ../../foo/bar.txt
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


/// finds the closest repo given a directory
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
                fname == ".art" && p.is_dir()
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


#[cfg(test)]
mod tests {
    use super::*;
    use test_data;
    #[test]
    fn test_find_repo() {
        let simple = &test_data::TSIMPLE_DIR;
        assert_eq!(find_repo(simple.as_path()).unwrap(), simple.as_path());
        assert_eq!(find_repo(simple.join("design").join("lvl_1").as_path()).unwrap(),
                   simple.as_path());
        assert!(find_repo(env::temp_dir().as_path()).is_none());
    }
}
