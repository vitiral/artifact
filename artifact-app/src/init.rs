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

use artifact_data::*;
use crate::dev_prelude::*;
use termstyle::Color::*;
use termstyle::{self, Color, El, Table, Text};

#[derive(Debug, StructOpt)]
#[structopt(name = "init", about = "Initialize the directory for artifact.")]
pub struct Init {
    #[structopt(long = "verbose", short = "v", default_value = "0")]
    /// Pass many times for more log output.
    pub verbosity: u64,

    #[structopt(long = "work-dir")]
    /// Use a different working directory [default: $CWD]
    pub work_dir: Option<String>,
}

const DESIGN_DIR: &str = "design";
const PURPOSE_MD: &str = "purpose.md";
const INIT_SETTINGS_TOML: &str = include_str!("data/settings.toml");
const INIT_PURPOSE_MD: &str = include_str!("data/purpose.md");

/// SPC-cli.init
pub fn run(cmd: Init) -> Result<i32> {
    set_log_verbosity!(cmd);
    let work_dir = work_dir!(cmd);
    info!(
        "Running art-init in working directory {}",
        work_dir.display()
    );

    let art = work_dir.join(ART_DIR);
    ensure!(
        !art.exists(),
        "{} directory already exists at {}",
        ART_DIR,
        work_dir.display()
    );
    let art = PathDir::create(art)?;
    let settings = PathFile::create(art.join(SETTINGS_FILE))?;
    println!("Created settings file at: {}", settings.display());
    settings.write_str(INIT_SETTINGS_TOML)?;
    let design = PathDir::create(work_dir.join(DESIGN_DIR))?;
    let purpose = PathFile::create(design.join(PURPOSE_MD))?;
    println!("Created initial purpose at: {}", purpose.display());
    purpose.write_str(INIT_PURPOSE_MD)?;
    println!(
        "Successfully initialized artifact project: {}",
        work_dir.display()
    );
    Ok(0)
}
