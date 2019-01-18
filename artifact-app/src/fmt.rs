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
//! Format the project

use crate::dev_prelude::*;
use artifact_data::*;

#[derive(Debug, StructOpt)]
#[structopt(name = "fmt")]
/// Format the project and change the filetype.
pub struct Fmt {
    #[structopt(long = "verbose", short = "v", default_value = "0")]
    /// Pass many times for more log output.
    pub verbosity: u64,

    #[structopt(long = "work-dir")]
    /// Use a different working directory [default: $CWD]
    pub work_dir: Option<String>,

    #[structopt(long = "type")]
    /// Set the type of all files
    pub ty_: Option<String>,
}

/// #SPC-cli.fmt
pub fn run(cmd: Fmt) -> Result<i32> {
    set_log_verbosity!(cmd);
    let repo = find_repo(&work_dir!(cmd))?;
    info!("Running art-fmt in repo {}", repo.display());
    modify_project(&repo, Vec::new())?;
    Ok(0)
}
