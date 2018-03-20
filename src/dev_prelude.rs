/* artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018  Garrett Berg <@vitiral, vitiral@gmail.com>
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
pub use expect_macro::*;

#[allow(unused_imports)]
pub use ergo::*;
#[allow(unused_imports)]
pub use quicli::prelude::*;
pub use indexmap::*;
pub use artifact_data::ART_DIR;
pub use artifact_lib::*;

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
    }.to_level_filter();

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
