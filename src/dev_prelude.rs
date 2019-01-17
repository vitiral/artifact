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
pub use expect_macro::*;

pub use artifact_data::ART_DIR;
pub use artifact_lib::*;
#[allow(unused_imports)]
pub use ergo::*;
#[allow(unused_imports)]
pub use quicli::prelude::*;

#[macro_export]
macro_rules! work_dir { [$cmd:expr] => {{
    match $cmd.work_dir {
        Some(ref d) => PathDir::new(d),
        None => PathDir::current_dir(),
    }?
}}}

#[macro_export]
macro_rules! set_log_verbosity { [$cmd:expr] => {{
    set_log_verbosity("art", $cmd.verbosity)?;
}}}

/// Set the logs verbosity based on an integer value:
///
/// - `0`: error
/// - `1`: warn
/// - `2`: info
/// - `3`: debug
/// - `>=4`: trace
///
/// This is used in the [`main!`] macro. You should typically use that instead.
///
/// [`main!`]: macro.main.html
pub fn set_log_verbosity(pkg: &str, verbosity: u64) -> Result<()> {
    let log_level = match verbosity {
        0 => LogLevel::Error,
        1 => LogLevel::Warn,
        2 => LogLevel::Info,
        3 => LogLevel::Debug,
        _ => LogLevel::Trace,
    }
    .to_level_filter();

    LoggerBuilder::new()
        .filter(Some(pkg), log_level)
        .filter(None, LogLevel::Warn.to_level_filter())
        .try_init()?;
    Ok(())
}

/// Find the project repo directory.
pub fn find_repo(initial: &PathDir) -> Result<PathDir> {
    let mut dir = initial.clone();
    loop {
        if dir.join(ART_DIR).exists() {
            return Ok(dir);
        }
        dir = match dir.parent_dir() {
            Some(d) => d,
            None => bail!("{} not within a directory with `.art`", initial.display()),
        };
    }
}
