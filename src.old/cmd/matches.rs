/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

use dev_prefix::*;
use super::types::*;
use super::export;
use super::init;
use super::tutorial;
use super::ls;
use super::check;
use super::fmt as fmtcmd;
use super::update;
use super::server;
#[cfg(feature = "beta")]
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
        .subcommand(server::get_subcommand());

    add_beta_cmds(app)
}

pub fn get_matches<'a, I, T>(args: I) -> ClapResult<ArgMatches<'a>>
where
    I: IntoIterator<Item = T>,
    T: Into<OsString> + clone::Clone,
{
    let app = art_app();
    app.get_matches_from_safe(args)
}

#[cfg(feature = "beta")]
/// add any beta cmdline features here
fn add_beta_cmds<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app.subcommand(plugin::get_subcommand())
}

#[cfg(not(feature = "beta"))]
/// add any beta cmdline features in the `[#cfg(feature = "beta")]` function
fn add_beta_cmds<'a, 'b>(app: App<'a, 'b>) -> App<'a, 'b> {
    app
}
