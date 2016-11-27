
use super::types::*;
use super::fmt as cmdfmt;
use super::super::core::{ArtifactData, Artifact};

#[cfg(feature = "web-api")]
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


/// pull out the command settings
pub fn get_cmd(matches: &ArgMatches) -> String {
    matches.value_of("addr").unwrap_or("127.0.0.1:4000").to_string()
}

#[cfg(feature = "web-api")]
pub fn run_server(artifacts: &Artifacts, addr: &str) {
    let data: Vec<ArtifactData> = artifacts
        .iter().map(|(name, model)| model.to_data(&name)).collect();
    api::start_api(data, addr);
}

#[cfg(not(feature = "web-api"))]
pub fn run_server() {
    println!("this instance of rst was not compiled with the server enabled");
}
