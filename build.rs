#[cfg(feature = "web")] extern crate tar;
extern crate serde_codegen;

pub use std::env;
pub use std::path::Path;
pub use std::process::Command;

fn main() {
    let out_dir = env::var_os("OUT_DIR").unwrap();

    let src = Path::new("src/core/serde_types.in.rs");
    let dst = Path::new(&out_dir).join("serde_types.rs");
    serde_codegen::expand(&src, &dst).unwrap();
}
