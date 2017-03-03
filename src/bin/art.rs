/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2017  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */
extern crate artifact_app;
extern crate error_chain;
use std::io;
use std::env;
use std::process;

use artifact_app::cmd;

fn main() {
    let rc = match cmd::cmd(&mut io::stdout(), env::args()) {
        Err(e) => {
            use std::io::Write;
            let stderr = &mut io::stderr();
            let errmsg = "Error writing to stderr";

            writeln!(stderr, "Encountered Error:\n").expect(errmsg);

            let mut was_caused = false;
            for e in e.iter().skip(1) {
                was_caused = true;
                writeln!(stderr, "## caused by: {}", e).expect(errmsg);
            }
            if was_caused {
                println!("Error was:")
            }
            println!("{}", e);
            1
        }
        Ok(rc) => rc,
    };
    process::exit(rc as i32)
}
