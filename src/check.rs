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
//! Check for errors

use artifact_data::*;
use dev_prelude::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "check")]
/// Check your project for errors and warnings.
pub struct Check {
    #[structopt(long = "verbose", short = "v", default_value = "0")]
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
