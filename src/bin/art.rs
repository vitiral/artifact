extern crate artifact_app;
use std::process::exit;

fn main() {
    match artifact_app::run() {
        Ok(rc) => exit(rc),
        Err(e) => {
            eprintln!("{}", e);
            exit(1);
        }
    }
}
