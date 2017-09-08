
use super::types::*;

use types::{Project, ServeCmd};
use super::super::api;

/// Get the server subcommand for the cmdline
/// Partof:
pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("serve")
        .about(
            "Serve the web-ui and json-rpc backend\n
               The server is hosted at ADDRESS, which can also be a port number for
               localhost.\n
               If address is a non-port, the server is readonly by default.\n
               if address is a port number (localost) then the server is editable by default.\n
               Either can be overridden explicitly through the --readonly or --editable flags",
        )
        .settings(&SUBCMD_SETTINGS)
        .arg(
            Arg::with_name("address")
                .value_name("ADDRESS")
                .help("Port (localhost) or address to host the server at"),
        )
        .arg(
            Arg::with_name("readonly")
                .short("r")
                .long("readonly")
                .help("Host the server as readonly"),
        )
        .arg(
            Arg::with_name("editable").short("e").long("editable").help(
                "DANGEROUS: host the server as editable. This is dangerous when
                      not hosting on localhost as ANYONE with access will be able to
                      at least edit your design documents. NO GUARANTEES ARE MADE ABOUT
                      THE SECURITY OF THIS FEATURE.",
            ),
        )
        .arg(
            Arg::with_name("path_url")
                .long("path-url")
                .takes_value(true)
                .help(
                    "Use the given format for creating links to artifact definitions.\n\n
                      \texample: https://github.com/my/proj/tree/SOME_HASH/{path}#{line}
                      \nWhere SOME_HASH is a hash given by you and \"{path}\" and \"{line}\"
                      are special markers that artifact uses for creating the link
                      for each artifact.\n
                      This is allows for easy viewing of the code where artifacts \
                      are implemented.",
                ),
        )
}


/// get command settings
pub fn get_cmd(matches: &ArgMatches) -> ServeCmd {
    let (mut readonly, addr) = {
        let addr = matches.value_of("addr").unwrap_or("5373");

        if let Ok(port) = addr.parse::<u64>() {
            // localhost so readonly false by default
            (false, format!("127.0.0.1:{}", port))
        } else {
            // non-localhost so readonly true by default
            (true, addr.to_string())
        }
    };

    if matches.is_present("editable") {
        readonly = false;
    }
    if matches.is_present("readonly") {
        readonly = true;
    }

    ServeCmd {
        addr: addr,
        readonly: readonly,
        path_url: matches.value_of("path_url").unwrap_or("").to_string(),
    }
}

// TODO: should technically return result
// need to do conditional compilation on types
// to auto-convert web errors
pub fn run_cmd(project: Project, cmd: &ServeCmd) {
    debug!("running server with: {:?}", cmd);
    api::start_api(project, cmd);
}
