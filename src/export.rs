/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018 Rett Berg <@vitiral, vitiral@gmail.com>
 *
 * The source code is Licensed under either of
 *
 * * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
 *   http://www.apache.org/licenses/LICENSE-2.0)
 * * MIT license ([LICENSE-MIT](LICENSE-MIT) or
 *   http://opensource.org/licenses/MIT)
 *
 * at your option.
 *
 * Unless you explicitly state otherwise, any contribution intentionally submitted
 * for inclusion in the work by you, as defined in the Apache-2.0 license, shall
 * be dual licensed as above, without any additional terms or conditions.
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
                The type of value to export. Supported values: [html, md]\n"
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

fn write_markdown(cmd: &Export, project_ser: &ProjectSer) -> io::Result<()> {
    let mut out = FileEdit::create(&cmd.path)?;
    project_ser.to_markdown(&mut out)?;
    out.flush()?;
    Ok(())
}

/// SPC-cli.init
pub fn run(cmd: Export) -> Result<i32> {
    set_log_verbosity!(cmd);
    let repo = find_repo(&work_dir!(cmd))?;
    info!("Running art-export in repo {}", repo.display());

    let (_, project) = read_project(repo)?;

    let project_ser = project.to_ser();

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
                project: Some(project_ser),
                web_type: WebType::Static,
            };

            expect!(frontend::unpack_frontend(&dir, &init));
            Ok(0)
        },
        "md" => {
            match write_markdown(&cmd, &project_ser) {
                Ok(_) => Ok(0),
                Err(e) => {
                    eprintln!("ERROR: {}", e);
                    return Ok(1);
                },

            }
        },
        _ => {
            eprintln!("ERROR: unexpected type {:?}", cmd.ty);
            Ok(1)
        }
    }
}
