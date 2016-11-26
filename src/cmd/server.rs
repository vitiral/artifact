
use super::types::*;
use super::fmt as cmdfmt;
use super::super::api;

/// Get the server subcommand for the cmdline
/// Partof: 
pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("server")
        .about("start the web-ui server")
        .settings(&[AS::DeriveDisplayOrder, AS::ColoredHelp])
        .arg(Arg::with_name("addr")
                 .help("full address to start server on. Default='127.0.0.1:8000'")
                 .use_delimiter(false))
}

pub fn run_server() {
    api::start_api();
}
