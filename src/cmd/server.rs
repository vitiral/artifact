
use super::types::*;

use types::Project;
use super::super::api;

/// Get the server subcommand for the cmdline
/// Partof:
pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("serve")
        .about("serve the web-ui and json-rpc backend")
        .settings(&SUBCMD_SETTINGS)
        .arg(Arg::with_name("addr")
            .help("full address to start server on. Default='127.0.0.1:4000'")
            .use_delimiter(false)
            .required(false))
        .arg(Arg::with_name("edit")
            .long("edit")
            .short("e")
            .help("enable editing. ALPHA NOTICE: this feature is not yet \
                   secure. DO NOT USE ON NON TRUSTED NETWORK"))
}


#[derive(Debug)]
pub struct Cmd {
    pub addr: String,
    pub edit: bool,
}

/// pull out the command settings
pub fn get_cmd(matches: &ArgMatches) -> Cmd {
    Cmd {
        addr: matches.value_of("addr").unwrap_or("127.0.0.1:4000").to_string(),
        edit: matches.is_present("edit"),
    }
}

// TODO: should technically return result
// need to do conditional compilation on types
// to auto-convert web errors
pub fn run_cmd(project: Project, cmd: &Cmd) {
    debug!("running server: {:?}", cmd);
    api::start_api(project, &cmd.addr, cmd.edit);
}
