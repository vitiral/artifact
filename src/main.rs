extern crate regex;
extern crate walkdir;
extern crate toml;

#[macro_use]
extern crate lazy_static;

pub mod core;

fn main() {
    println!("importing");
    match core::load::recursive_raw_load("docs") {
        Ok(_) => println!("success"),
        Err(err) => println!("error: {}", err),
    }
}
