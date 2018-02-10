//! export command

use tar::Archive;

use dev_prefix::*;
use types::*;
use cmd::types::*;
use export;

const WEB_STATIC_TAR: &[u8] = include_bytes!("data/web-ui-static.tar");

const REPLACE_PATH: &str = "REPLACE_WITH_PATH_URL";
const REPLACE_ARTIFACTS: &str = "REPLACE_WITH_ARTIFACTS";

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    // #SPC-cmd-export
    SubCommand::with_name("export")
        .about("Export artifacts as static files")
        .settings(&SUBCMD_SETTINGS)
        .arg(Arg::with_name("type").help(
            "Type of export.\n- html: static html. Writes to `./index.html` and `./css/`",
        ))
        .arg(
            Arg::with_name("output")
                .short("o")
                .long("output")
                .value_name("PATH")
                .help(
                    "The export output directory. If omitted, the current working directory \
                     is used.",
                ),
        )
        .arg(
            Arg::with_name("path_url")
                .long("path-url")
                .takes_value(true)
                .help(
                    "\nUse the given format for creating links to artifact definitions when using \
                     `export html`.\
                     \nThis allows for easy viewing of the code where artifacts are implemented.\
                     \nExample:\n    $GITHUB_URL/blob/$(git rev-parse HEAD)/{path}#L{line}\
                     \n\n- `$REPO_URL` is the url for your repo on github\
                     \n  (i.e. https://github.com/vitiral/artifact)\
                     \n- `$(git rev-parse HEAD)` is a bash cmd which finds\
                     \n  the current hash of your repo\
                     \n- `{path}` and `{line}` are special markers that\
                     \n  artifact uses for creating the link for each artifact.\
                     \n\nThis results in links like the following being automatically placed in \
                     your \"code location\" fields:\
                     \nhttps://github.com/vitiral/artifact/blob/\
                     4688c950731b081387976bbaf3fb0dc5612d5cad/src/cmd/ls.rs#L28\n",
                ),
        )
}

#[derive(Debug)]
pub enum ExportType {
    Html,
}

#[derive(Debug)]
pub struct Cmd<'a> {
    ty: ExportType,
    output: Option<&'a Path>,
    path_url: String,
}

pub fn get_cmd<'a>(matches: &'a ArgMatches) -> Result<Cmd<'a>> {
    let ty = match matches
        .value_of("type")
        .unwrap_or("")
        .to_ascii_lowercase()
        .as_str()
    {
        "html" => ExportType::Html,
        t => return Err(ErrorKind::CmdError(format!("unknown type: {}", t)).into()),
    };
    let dir = matches.value_of("output").map(|x| Path::new(x).as_ref());
    Ok(Cmd {
        ty: ty,
        output: dir,
        path_url: matches.value_of("path_url").unwrap_or("").to_string(),
    })
}

pub fn run_cmd(cwd: &Path, project: &Project, cmd: &Cmd) -> Result<u8> {
    let output = cmd.output.unwrap_or(cwd);
    debug!("running with cmd: {:?}", cmd);
    match cmd.ty {
        ExportType::Html => {
            // get the artifacts as json and replace with escaped chars
            let data = export::project_artifacts_to_json(project, None)
                .replace("\\", "\\\\")
                .replace("'", "\\'");

            // unpack the index.html + css/ files
            let mut archive = Archive::new(WEB_STATIC_TAR);
            if let Err(e) = fs::remove_dir_all(output.join("css")) {
                if e.kind() == io::ErrorKind::NotFound {
                } else {
                    return Err(e.into());
                }
            }
            archive
                .unpack(&output)
                .expect("unable to unpack web frontend");
            let index_path = output.join("index.html");
            let mut index = fs::OpenOptions::new()
                .read(true)
                .write(true)
                .open(index_path)
                .expect("couldn't open app.js");
            let mut text = String::new();
            index
                .read_to_string(&mut text)
                .expect("index.html couldn't be read");

            // replace index.html to include the artifacts inline
            index.seek(SeekFrom::Start(0)).unwrap();
            index.set_len(0).unwrap(); // delete what is there
            index
                .write_all(
                    text.replace(REPLACE_ARTIFACTS, &data)
                        .replace(REPLACE_PATH, &cmd.path_url)
                        .as_bytes(),
                )
                .unwrap();
            index.flush().unwrap();
        }
    }
    Ok(0)
}
