/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use self_update;

use cmd::types::*;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("update")
        .about("Update to a different version (default newest)")
        .settings(&SUBCMD_SETTINGS)
        .arg(
            Arg::with_name("version")
                .value_name("VERSION")
                .use_delimiter(false)
                .help("Version to update to. Default=latest version"),
        )
}

#[derive(Debug)]
pub struct Cmd {
    version: Option<String>,
}


/// get all the information from the user input
pub fn get_cmd(matches: &ArgMatches) -> Cmd {
    Cmd {
        version: matches.value_of("version").map(|s| s.to_string()),
    }
}

pub fn run_cmd(cmd: &Cmd) -> ::std::result::Result<u8, Box<::std::error::Error>> {
    let target = self_update::get_target()?;
    let mut builder = self_update::backends::github::UpdateBuilder::new()?;
    builder
        .repo_owner("vitiral")
        .repo_name("artifact")
        .target(&target)
        .bin_name("art")
        .show_download_progress(true)
        .current_version(cargo_crate_version!());

    if let Some(ref v) = cmd.version {
        builder.target_version_tag(v);
    }

    let status = builder.build()?.update()?;
    println!("Update status: `{}`!", status.version());
    Ok(0)
}
