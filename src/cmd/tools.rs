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

use clap::{App, Shell};
use std::fs;
use dev_prefix::*;
use types::*;
use cmd::types::*;
use cmd::matches::art_app;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("tools")
        .settings(&SUBCMD_SETTINGS)
        .about(
            "Command for integrating with external tooling, like generating shell completions",
        )
        .arg(
            Arg::with_name("value")
                .required(true)
                .possible_values(&[
                    "bash-completions",
                    "fish-completions",
                    "zsh-completions",
                    "psh-completions",
                ])
                .help(
                    "Options:\n\
                     - bash-completions: Generates bash completions\n\
                     - fish-completions: Generages fish completions\n\
                     - zsh-completions: Generates zsh completions\n\
                     - ps-completions: Generages PowerShell completions\n",
                ),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .value_name("PATH")
                .help(
                    "Path of file to place the output of the tools command \
                     defaults to standard output otherwise.",
                ),
        )
}

#[derive(Debug)]
enum Tool {
    Comp(Shell),
}

#[derive(Debug)]
pub struct Cmd<'a> {
    tool: Tool,
    output: Option<&'a Path>,
}

pub fn get_cmd<'a>(matches: &'a ArgMatches) -> Result<Cmd<'a>> {
    let tool = match matches
        .value_of("value")
        .expect("clap error in argument parsing!")
    {
        "bash-completions" => Tool::Comp(Shell::Bash),
        "fish-completions" => Tool::Comp(Shell::Fish),
        "zsh-completions" => Tool::Comp(Shell::Zsh),
        "ps-completions" => Tool::Comp(Shell::PowerShell),
        _ => panic!("clap error in argumen parsing!"),
    };
    let out = matches.value_of("output").map(|p| Path::new(p).as_ref());
    Ok(Cmd {
        tool: tool,
        output: out,
    })
}

pub fn run_cmd<W: io::Write>(cmd: &Cmd, w: W) -> Result<u8> {
    match cmd.tool {
        Tool::Comp(_) => if let Some(path) = cmd.output {
            let md = fs::metadata(path);
            if md.ok().map_or(false, |md| md.is_dir()) {
                run_cmd_completions(cmd, w, true)
            } else {
                // we can't see it as a directory, try to create it as
                // a file.
                let file = fs::OpenOptions::new()
                    .write(true)
                    .truncate(true)
                    .create(true)
                    .open(path)?;
                run_cmd_completions(cmd, file, false)
            }
        } else {
            run_cmd_completions(cmd, w, false)
        },
    }
}

fn run_cmd_completions<W: io::Write>(cmd: &Cmd, mut w: W, is_dir: bool) -> Result<u8> {
    // If use_gen_to is true then we use gen_completions_to as our output
    // function, otherwise we use gen_completions. This is only relevant
    // for completion generating options.
    match cmd.tool {
        Tool::Comp(shell) => {
            let (sh, name) = match shell {
                Shell::Bash => ("bash", "art.bash-completion"),
                Shell::Fish => ("fish", "art.fish"),
                Shell::Zsh => ("zsh", "_art"),
                Shell::PowerShell => ("PowerShell", "_art.ps1"),
            };
            if !is_dir {
                info!(
                    "Generating {} completions to {}",
                    sh,
                    cmd.output.unwrap_or("standard output".as_ref()).display(),
                );
                art_app().gen_completions_to("art", shell, &mut w)
            } else {
                // unwrap is safe as cmd.output had to be Some for this branch to
                // be taken
                info!(
                    "Generating {} completions to {}",
                    sh,
                    cmd.output.unwrap().join(name).display(),
                );
                art_app().gen_completions("art", shell, cmd.output.unwrap())
            }
            Ok(0)
        }
    }
}
