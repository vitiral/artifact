/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */

extern crate artifact_app;
use std::io;
use std::env;
use std::process;

use artifact_app::cmd;

fn main() {
    cmd::cmd(&mut io::stdout(), env::args()).unwrap();
    process::exit(i32::from(0))
    // let rc = match cmd::cmd(&mut io::stdout(), env::args()) {
    //     Err(e) => {
    //         eprintln!("Encountered Error:\n");

    //         let mut was_caused = false;
    //         for e in e.iter().skip(1) {
    //             was_caused = true;
    //             eprintln!("## caused by: {}", e);
    //         }
    //         if was_caused {
    //             eprintln!("Error was:")
    //         }
    //         eprintln!("{}", e);
    //         1
    //     }
    //     Ok(rc) => rc,
    // };
    // process::exit(i32::from(rc))
}
