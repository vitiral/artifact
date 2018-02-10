pub use expect_macro::*;

#[allow(unused_imports)]
pub use ergo::*;
#[allow(unused_imports)]
pub use quicli::prelude::*;
pub use ordermap::*;

#[macro_export]
macro_rules! work_dir { [$cmd:expr] => {{
    match $cmd.work_dir {
        Some(d) => PathDir::new(d),
        None => PathDir::current_dir(),
    }?
}}}
