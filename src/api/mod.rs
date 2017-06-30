use dev_prefix::*;

use std::sync::Mutex;

use nickel::{Request, Response, MiddlewareResult, Nickel, HttpRouter, MediaType,
             StaticFilesHandler};
use nickel::status::StatusCode;
use serde_json;

use tar::Archive;
use tempdir::TempDir;

use types::{ServeCmd, Project};
use export::ArtifactData;

mod constants;
pub mod utils;
mod handler;
mod crud;

#[cfg(test)]
mod tests;

const WEB_FRONTEND_TAR: &'static [u8] = include_bytes!("data/web-ui.tar");
const REPLACE_FLAGS: &str = "{/* REPLACE WITH FLAGS */}";

pub struct LockedData {
    cmd: ServeCmd,
    artifacts: Vec<ArtifactData>,
    project: Project,
}

lazy_static! {
    static ref LOCKED: Mutex<LockedData> = Mutex::new(
        LockedData {
            cmd: ServeCmd::default(),
            artifacts: Vec::new(),
            project: Project::default(),
        }
    );
}

fn setup_headers(res: &mut Response) {
    let head = res.headers_mut();
    let bv = |s: &str| Vec::from(s.as_bytes());
    head.set_raw("Access-Control-Allow-Origin", vec![bv("*")]);
    head.set_raw(
        "Access-Control-Allow-Methods",
        vec![bv("GET, POST, OPTIONS, PUT, PATCH, DELETE")],
    );
    head.set_raw(
        "Access-Control-Allow-Headers",
        vec![bv("X-Requested-With,content-type")],
    );
}

fn config_json_res(res: &mut Response) {
    res.set(MediaType::Json);
    res.set(StatusCode::Ok);
}

fn handle_artifacts<'a>(req: &mut Request, mut res: Response<'a>) -> MiddlewareResult<'a> {
    setup_headers(&mut res);
    debug!("handling json-rpc request");

    let mut body = vec![];
    req.origin.read_to_end(&mut body).unwrap();
    let body = match str::from_utf8(&body) {
        Ok(b) => b,
        Err(e) => {
            res.set(StatusCode::BadRequest);
            return res.send(format!("invalid utf8: {:?}", e));
        }
    };

    debug!("body: {:?}", body);
    match handler::RPC_HANDLER.handle_request_sync(body) {
        Some(body) => {
            trace!("- response {}", body);
            config_json_res(&mut res);
            res.send(body)
        }
        None => {
            let msg = "InternalServerError: Got None from json-rpc handler";
            error!("{}", msg);
            res.set(StatusCode::InternalServerError);
            res.send(msg)
        }
    }
}

fn handle_options<'a>(_: &mut Request, mut res: Response<'a>) -> MiddlewareResult<'a> {
    setup_headers(&mut res);
    res.set(StatusCode::Ok);
    res.send("ok")
}

/// host the frontend web-server, returning the tempdir where it the
/// static files are being held. It is important that this tempdir
/// always be owned, ortherwise the files will be deleted!
fn host_frontend(server: &mut Nickel, cmd: &ServeCmd) -> TempDir {
    // it is important that tmp_dir never goes out of scope
    // or the webapp will be deleted!
    let tmp_dir = TempDir::new("artifact-web-ui").expect("unable to create temporary directory");
    let dir = tmp_dir.path().to_path_buf(); // we have to clone this because *borrow*
    info!("unpacking web-ui at: {}", dir.display());

    let mut archive = Archive::new(WEB_FRONTEND_TAR);
    archive.unpack(&dir).expect("unable to unpack web frontend");

    // replace the default ip address with the real one
    let app_js_path = dir.join("app.js");
    let mut app_js = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(app_js_path)
        .expect("couldn't open app.js");
    let mut text = String::new();
    app_js
        .read_to_string(&mut text)
        .expect("app.js couldn't be read");
    app_js.seek(SeekFrom::Start(0)).unwrap();
    app_js.set_len(0).unwrap(); // delete what is there
    // the elm app uses a certain address by default, replace it

    assert!(text.contains(REPLACE_FLAGS));
    app_js
        .write_all(
            text.replace(REPLACE_FLAGS, &serde_json::to_string(cmd).unwrap())
                .as_bytes(),
        )
        .unwrap();
    app_js.flush().unwrap();

    server.utilize(StaticFilesHandler::new(&dir));
    println!("hosting web ui at {}", cmd.addr);
    tmp_dir
}

/// start the json-rpc API server
#[allow(unused_variables)] // need to hold ownership of tmp_dir
pub fn start_api(project: Project, cmd: &ServeCmd) {
    // store artifacts and files into global mutex

    {
        let mut artifacts = utils::convert_to_data(&project);
        let mut locked = LOCKED.lock().unwrap();
        let lref = locked.deref_mut();
        let compare_by = |a: &ArtifactData| a.name.to_ascii_uppercase();
        artifacts.sort_by(|a, b| compare_by(a).cmp(&compare_by(b)));
        lref.artifacts = artifacts;
        lref.project = project;
        lref.cmd = cmd.clone();
    }

    let endpoint = "/json-rpc";
    let mut server = Nickel::new();

    server.get(endpoint, handle_artifacts);
    server.put(endpoint, handle_artifacts);
    server.options(endpoint, handle_options);

    // host the frontend files using a static file handler
    // and own the tmpdir for as long as needed
    let tmp_dir = host_frontend(&mut server, cmd);

    server.listen(&cmd.addr).expect("cannot connect to port");
}
