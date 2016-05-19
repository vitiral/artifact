extern crate regex;
extern crate walkdir;
// $LOC-core-loading-toml
extern crate toml;
extern crate strfmt;

#[macro_use]
extern crate lazy_static;

pub mod core;

fn main() {
    println!("importing");
    // match core::load::recursive_raw_load("docs") {
    //     Ok(_) => println!("success"),
    //     Err(err) => println!("error: {}", err),
    // }
}
