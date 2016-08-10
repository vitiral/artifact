extern crate rst_app;
use std::io;
use std::env;

use rst_app::cmd;
fn main() {
    cmd::cmd(&mut io::stdout(), env::args());
}
