/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2017  Garrett Berg <@vitiral, vitiral@gmail.com>
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
                .help("Version to update to. Default is latest."),
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
