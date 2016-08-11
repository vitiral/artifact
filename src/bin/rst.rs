extern crate rst_app;
use std::io;
use std::env;
use std::process;

use rst_app::cmd;
fn main() {
    process::exit(cmd::cmd(&mut io::stdout(), env::args()));
}
