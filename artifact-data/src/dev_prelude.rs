/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018 Rett Berg <@vitiral, vitiral@gmail.com>
 *
 * The source code is Licensed under either of
 *
 * * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
 *   http://www.apache.org/licenses/LICENSE-2.0)
 * * MIT license ([LICENSE-MIT](LICENSE-MIT) or
 *   http://opensource.org/licenses/MIT)
 *
 * at your option.
 *
 * Unless you explicitly state otherwise, any contribution intentionally submitted
 * for inclusion in the work by you, as defined in the Apache-2.0 license, shall
 * be dual licensed as above, without any additional terms or conditions.
 * */
pub(crate) use artifact_lib;
pub(crate) use artifact_lib::*;
pub(crate) use ergo::*;
pub(crate) use ergo::path_abs::ser::ToStfu8;
#[allow(unused_imports)]
pub(crate) use expect_macro::*;
pub(crate) use std::ffi::OsStr;
use std::fs;
use std::io;

pub(crate) use failure::Error;
pub(crate) use std::result;

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

fn create_dir_maybe<P: AsRef<Path>>(path: P) -> path_abs::Result<PathDir> {
    let arc = PathSer::from(path.as_ref());
    fs::create_dir(&arc).map_err(|err| path_abs::Error::new(err, "creating dir", arc.clone().into()))?;
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
                    from.copy(expect!(to.concat(to_postfix))).map_err(|err| err.into()),
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
                if let Err(err) = PathDir::create(expect!(to.concat(to_postfix))) {
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
