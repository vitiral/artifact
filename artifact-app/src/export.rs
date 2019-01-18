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

use crate::dev_prelude::*;
use artifact_data::*;
use std::io;

use crate::frontend;

#[derive(Debug, StructOpt)]
#[structopt(
    name = "export",
    about = "\
Export artifacts in some format.

## Settings (.art/settings.toml)
The following can be added to control export's behavior

### md_family
Controls how the family is exported.

md_family = {
    type = \"list|dot\",
}
"
)]
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

fn export_html(cmd: &Export, project_ser: ProjectSer) -> io::Result<()> {
    let dir = PathDir::create(&cmd.path)?;
    let init = ProjectInitialSer {
        project: Some(project_ser),
        web_type: WebType::Static,
    };
    frontend::unpack_frontend(&dir, &init)?;
    Ok(())
}

fn export_markdown(cmd: &Export, project_ser: ProjectSer) -> io::Result<()> {
    use artifact_ser::markdown::*;
    let settings = SerMarkdownSettings {
        code_url: project_ser.settings.code_url.clone(),
        family: project_ser.settings.export.md_family.clone(),
        dot: project_ser.settings.export.md_dot.clone(),
    };
    let md = SerMarkdown::with_settings(&project_ser, settings);

    let mut out = FileEdit::create(&cmd.path)?;
    md.to_markdown(&mut out)?;
    out.flush()?;
    Ok(())
}

lazy_static! {
    static ref REPLACE_TEXT_RE: Regex = expect!(Regex::new(
        r#"(?xim)
        (?:^```dot\s*\n(?P<dot>[\s\S]+?\n)```$)  # graphviz dot rendering
        "#,
    ));
}

// fn replace_markdown<'t>(markdown: &'t str) -> Cow<'t, str> {
//     let replacer = |cap: &::ergo_std::regex::Captures| -> String {
//         if let Some(dot) = cap.name("dot") {
//             replace_markdown_dot(dot.as_str())
//         } else {
//             panic!("Got unknown match in md: {:?}", cap);
//         }
//     };
//     REPLACE_TEXT_RE.replace_all(markdown, replacer)
// }
//
// fn replace_markdown_dot(dot: &str) -> String {
//     let html = graph::dot_html_string(dot);
//     format!("\n<html>\n{0}\n</html>\n", html)
// }

/// SPC-cli.init
pub fn run(cmd: Export) -> Result<i32> {
    set_log_verbosity!(cmd);
    let repo = find_repo(&work_dir!(cmd))?;
    info!("Running art-export in repo {}", repo.display());

    let (_, project) = read_project(repo)?;
    let project_ser = project.to_ser();

    let result = match cmd.ty.to_ascii_lowercase().as_str() {
        "html" => export_html(&cmd, project_ser),
        "md" => export_markdown(&cmd, project_ser),
        _ => {
            eprintln!("ERROR: unexpected type {:?}", cmd.ty);
            return Ok(1);
        }
    };

    match result {
        Ok(_) => Ok(0),
        Err(e) => {
            eprintln!("ERROR: {}", e);
            return Ok(1);
        }
    }
}
