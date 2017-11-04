/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use clap::{App, Shell};
use std::fs;
use dev_prefix::*;
use types::*;
use cmd::types::*;
use cmd::matches::art_app;

const BASH_COMPLETIONS: &str = "bash-completions";
const FISH_COMPLETIONS: &str = "fish-completions";
const ZSH_COMPLETIONS: &str = "zsh-completions";
const PS_COMPLETIONS: &str = "ps-completions";

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("plugin")
        .settings(&SUBCMD_SETTINGS)
        .about(
            "Command for integrating with external plugins, currently only \
             supports generating shell completions",
        )
        .arg(
            Arg::with_name("name")
                .required(true)
                .possible_values(&[
                    BASH_COMPLETIONS,
                    FISH_COMPLETIONS,
                    ZSH_COMPLETIONS,
                    PS_COMPLETIONS,
                ])
                .help("Plugin name"),
        )
        .arg(
            Arg::with_name("output")
                .long("output")
                .short("o")
                .value_name("PATH")
                .help(
                    "Path of file to place the output of the plugin command \
                     (defaults to standard output)",
                ),
        )
}

#[derive(Debug)]
enum Plugin {
    Comp(Shell),
}

#[derive(Debug)]
pub struct Cmd<'a> {
    plugin: Plugin,
    output: Option<&'a Path>,
}

pub fn get_cmd<'a>(matches: &'a ArgMatches) -> Result<Cmd<'a>> {
    let plugin = match matches
        .value_of("name")
        .expect("clap error in argument parsing!")
    {
        BASH_COMPLETIONS => Plugin::Comp(Shell::Bash),
        FISH_COMPLETIONS => Plugin::Comp(Shell::Fish),
        ZSH_COMPLETIONS => Plugin::Comp(Shell::Zsh),
        PS_COMPLETIONS => Plugin::Comp(Shell::PowerShell),
        _ => panic!("clap error in argument parsing!"),
    };
    let out = matches.value_of("output").map(|p| Path::new(p).as_ref());
    Ok(Cmd {
        plugin: plugin,
        output: out,
    })
}

pub fn run_cmd<W: io::Write>(cmd: &Cmd, w: W) -> Result<u8> {
    match cmd.plugin {
        Plugin::Comp(_) => if let Some(path) = cmd.output {
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
    match cmd.plugin {
        Plugin::Comp(shell) => {
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
                    cmd.output
                        .unwrap_or_else(|| "standard output".as_ref())
                        .display(),
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
