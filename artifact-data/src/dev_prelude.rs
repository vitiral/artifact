/*
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
pub(crate) use ergo::*;
#[allow(unused_imports)]
pub(crate) use expect_macro::*;
// TODO: move these to std_prelude
pub(crate) use std::ffi::OsStr;
pub use std::cmp::Ord;
pub use std::cmp::PartialOrd;
pub use std::hash::{Hash, Hasher};
use std::io;
use std::fs;
pub(crate) use artifact_lib::*;
pub(crate) use artifact_lib;

pub(crate) use indexmap::{IndexMap, IndexSet};

pub(crate) use std::result;
pub(crate) use failure::Error;

pub(crate) type Result<V> = result::Result<V, Error>;

#[allow(dead_code)]
/// A simple implementation of "touch"
pub(crate) fn touch<P: AsRef<Path>>(path: P) -> ::std::io::Result<()> {
    OpenOptions::new()
        .create(true)
        .write(true)
        .open(path.as_ref())?;
    Ok(())
}

#[test]
fn sanity_trim_right() {
    let mut result = "  hello    ".into();
    string_trim_right(&mut result);
    assert_eq!(result, "  hello");
}

fn create_dir_maybe<P: AsRef<Path>>(path: P) -> path_abs::Result<PathDir> {
    let arc = PathArc::new(path);
    fs::create_dir(&arc).map_err(|err| path_abs::Error::new(err, "creating dir", arc.clone()))?;
    PathDir::new(arc)
}

/// Do a deep copy of a directory from one location to another.
pub fn deep_copy<P: AsRef<Path>>(send_err: Sender<io::Error>, from: PathDir, to: P) {
    let to = ch_try!(
        send_err,
        create_dir_maybe(to).map_err(|err| err.into()),
        return
    );

    let (send_file, recv_file) = ch::bounded(128);

    // First thread walks and creates directories, and sends files to copy
    take!(=send_err as errs, =to as to_walk);
    spawn(move || {
        walk_and_create_dirs(from, to_walk, errs, send_file);
    });

    // Threadpool copy files into directories that are pre-created.
    for _ in 0..num_cpus::get() {
        take!(=send_err, =recv_file, =to);
        spawn(move || {
            for (from, to_postfix) in recv_file {
                ch_try!(
                    send_err,
                    from.copy(to.join(to_postfix)).map_err(|err| err.into()),
                    continue
                );
            }
        });
    }
}

/// Do a contents-first yeild and follow any symlinks -- we are doing an _actual_ copy
fn walk_and_create_dirs(
    from: PathDir,
    to: PathDir,
    send_err: Sender<io::Error>,
    send_file: Sender<(PathFile, PathBuf)>,
) {
    let mut it = from.walk().follow_links(true).into_iter();
    loop {
        let entry = match it.next() {
            Some(entry) => entry,
            None => break,
        };
        macro_rules! handle_err {
            ($entry:expr) => {
                match $entry {
                    Ok(e) => e,
                    Err(err) => {
                        ch!(send_err <- err.into());
                        continue;
                    }
                }
            };
        }
        let entry = handle_err!(entry);
        let to_postfix = expect!(
            entry.path().strip_prefix(&from),
            "{} does not have prefix {}",
            entry.path().display(),
            from.display()
        );
        match handle_err!(PathType::new(entry.path())) {
            PathType::Dir(_) => {
                // Create it immediately
                if let Err(err) = PathDir::create(to.join(to_postfix)) {
                    ch!(send_err <- err.into());
                    // We couldn't create the directory so it needs to be skipped.
                    it.skip_current_dir();
                }
            }
            PathType::File(from_file) => {
                ch!(send_file <- (from_file, to_postfix.to_path_buf()));
            }
        }
    }
}
