/* artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018  Garrett Berg <@vitiral, vitiral@gmail.com>
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

use artifact_data::*;
use dev_prelude::*;
use std::io;

use frontend;

#[derive(Debug, StructOpt)]
#[structopt(name = "export", about = "Export artifacts in some format.")]
pub struct Export {
    #[structopt(long = "verbose", short = "v", default_value = "0")]
    /// Pass many times for more log output.
    pub verbosity: u64,

    #[structopt(long = "work-dir")]
    /// Use a different working directory [default: $CWD]
    pub work_dir: Option<String>,

    #[structopt(
        name = "TYPE",
        help = "\
                The type of value to export. Supported values: [html]\n"
    )]
    pub ty: String,

    #[structopt(
        name = "PATH",
        help = "\
                The destination to export the data to.\n\n\
                html: this will be the directory that is created.\n"
    )]
    pub path: String,
}

/// SPC-cli.init
pub fn run(cmd: Export) -> Result<i32> {
    let mut w = io::stdout();

    set_log_verbosity!(cmd);
    let repo = find_repo(&work_dir!(cmd))?;
    info!("Running art-export in repo {}", repo.display());

    let (_, project) = read_project(repo)?;

    match cmd.ty.to_ascii_lowercase().as_str() {
        "html" => {
            let dir = match PathDir::create(&cmd.path) {
                Ok(d) => d,
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                    return Ok(1);
                },
            };
            let init = ProjectInitialSer {
                project: Some(project.to_ser()),
                web_type: WebType::Static,
            };

            expect!(frontend::unpack_frontend(&dir, &init));
            Ok(0)
        },
        _ => {
            eprintln!("ERROR: unexpected type {:?}", cmd.ty);
            Ok(1)
        }
    }
}
