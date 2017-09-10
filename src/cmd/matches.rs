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

use dev_prefix::*;
use super::types::*;
use super::export;
use super::init;
use super::tutorial;
use super::ls;
use super::check;
use super::fmt as fmtcmd;
use super::update;
use super::plugin;

pub fn art_app<'a, 'b>() -> App<'a, 'b> {
    let app = App::new("artifact")
        .version(env!("CARGO_PKG_VERSION"))
        .about(
            "The requirements tracking tool made for developers. \
             Call `art tutorial` for a tutorial",
        )
        .author("https://github.com/vitiral/artifact")
        .settings(&APP_SETTINGS)
        .arg(
            Arg::with_name("verbose")
                .short("v")
                .multiple(true)
                .help("Verbose, pass up to 3 times to increase the level")
                .global(true),
        )
        .arg(
            Arg::with_name("quiet")
                .short("q")
                .long("quiet")
                .help("If set no output will be printed")
                .global(true),
        )
        .arg(
            Arg::with_name("work-tree")
                .long("work-tree")
                .value_name("PATH")
                .help("Use a different working tree instead of cwd")
                .takes_value(true)
                .global(true),
        )
        .subcommand(tutorial::get_subcommand())
        .subcommand(init::get_subcommand())
        .subcommand(ls::get_subcommand())
        .subcommand(check::get_subcommand())
        .subcommand(fmtcmd::get_subcommand())
        .subcommand(export::get_subcommand())
        .subcommand(update::get_subcommand())
        .subcommand(plugin::get_subcommand());

    app
}

pub fn get_matches<'a, I, T>(args: I) -> ClapResult<ArgMatches<'a>>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + clone::Clone,
{
    let app = art_app().subcommand(::cmd::server::get_subcommand());

    let app = add_beta_cmds(app);
    app.get_matches_from_safe(args)
}

#[cfg(feature = "beta")]
/// add any beta cmdline features here
pub fn add_beta_cmds<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app
}

#[cfg(not(feature = "beta"))]
/// add any beta cmdline features in the `[#cfg(feature = "beta")]` function
pub fn add_beta_cmds<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app
}
