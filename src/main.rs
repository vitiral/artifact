// # logger config
#[cfg(test)]      extern crate env_logger;
#[cfg(not(test))] extern crate fern;

// # general crates
extern crate itertools;
#[macro_use] extern crate lazy_static;
#[macro_use] extern crate log;

// # core crates
// LOC-core-loading-toml
// LOC-core-vars-lib
extern crate regex;
extern crate strfmt;
extern crate time;
extern crate toml;

// # ui-cmdline crates
extern crate clap;
extern crate ansi_term;

pub mod core;
mod cmdline;


fn main() {
    cmdline::cmd();
}
