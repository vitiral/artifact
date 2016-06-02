// # logger config
extern crate fern;

// # general crates
extern crate itertools;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

// # core crates
// SPC-core-load-toml
// SPC-core-vars-lib
extern crate regex;
extern crate strfmt;
extern crate time;
extern crate toml;

// # ui-cmdline crates
extern crate clap;
extern crate ansi_term;

pub mod core;
mod cmdline;

pub use cmdline::init_logger;

fn main() {
    cmdline::cmd();
}
