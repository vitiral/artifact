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
//! #SPC-cli.serve
use std::io;

use artifact_data::*;
use crate::dev_prelude::*;

mod handler;

#[derive(Debug, Default, Clone, StructOpt)]
#[structopt(name = "serve", about = "Serve the web-ui via http.")]
#[cfg_attr(rustfmt, rustfmt_skip)]
pub struct Serve {
    #[structopt(long = "verbose", short = "v", parse(from_occurrences))]
    /// Pass many times for more log output.
    pub verbosity: u64,

    #[structopt(long="work-dir")]
    /// Use a different working directory [default: $CWD]
    pub work_dir: Option<String>,

    /// Select the port to serve on.
    #[structopt(default_value="5373")]
    pub port: u64,
}

lazy_static! {
    static ref LOCKED: Mutex<Option<ProjectResult>> = Mutex::new(None);
}

/// Run the `art serve` command
pub fn run(cmd: Serve) -> Result<i32> {
    set_log_verbosity!(cmd);
    let repo = find_repo(&work_dir!(cmd))?;
    info!("Running art-serve in repo {}", repo.display());

    let (lints, project) = read_project(repo)?;
    {
        let mut locked = LOCKED.lock().unwrap();
        *locked = Some(ProjectResult {
            project: project,
            lints: lints,
        });
    }

    handler::start_api(cmd);
    Ok(0)
}
