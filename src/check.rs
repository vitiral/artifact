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
//! Check for errors

use dev_prelude::*;
use artifact_data::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "check")]
/// Check your project for errors and warnings.
pub struct Check {
    #[structopt(long = "verbose", short = "v", default_value="0")]
    /// Pass many times for more log output.
    pub verbosity: u64,

    #[structopt(long = "work-dir")]
    /// Use a different working directory [default: $CWD]
    pub work_dir: Option<String>,
}

/// #SPC-cli.check
pub fn run(cmd: Check) -> Result<i32> {
    set_log_verbosity!(cmd);
    let repo = find_repo(&work_dir!(cmd))?;
    info!("Running art-check in repo {}", repo.display());
    let (lints, _) = read_project(repo)?;
    if !lints.error.is_empty() {
        Err(lints.into())
    } else if !lints.is_empty() {
        eprintln!("{}", lints);
        Ok(2) // rc=2 if only warnings
    } else {
        Ok(0)
    }
}
