use dev_prefix::*;
use std::sync::Mutex;

// extern crate rustc_serialize;
// #[macro_use] extern crate nickel;
// #[macro_use] extern crate lazy_static;
// extern crate jsonrpc_core;

// #[macro_use] extern crate serde;
// #[macro_use] extern crate serde_derive;
// extern crate serde_json;

use nickel::{Request, Response, MiddlewareResult, Nickel, HttpRouter, MediaType};
use nickel::status::StatusCode;

use nickel::StaticFilesHandler;
use tar::Archive;
use tempdir::TempDir;

use core::{Project, ArtifactData};

mod constants;
mod utils;
mod handler;
mod update;

#[cfg(test)]
mod tests;

const WEB_FRONTEND_TAR: &'static [u8] = include_bytes!("data/web-ui.tar");

lazy_static! {
    static ref ARTIFACTS: Mutex<Vec<ArtifactData>> = Mutex::new(Vec::new());
    static ref PROJECT: Mutex<Project> = Mutex::new(Project::default());
}

fn setup_headers(res: &mut Response) {
    let head = res.headers_mut();
    let bv = |s: &str| Vec::from(s.as_bytes());
    head.set_raw("Access-Control-Allow-Origin", vec![bv("*")]);
    head.set_raw("Access-Control-Allow-Methods",
                 vec![bv("GET, POST, OPTIONS, PUT, PATCH, DELETE")]);
    head.set_raw("Access-Control-Allow-Headers",
                 vec![bv("X-Requested-With,content-type")]);
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

    trace!("request: {:?}", body);
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

/// host the frontend web-server, returning the tempdir where it the
/// static files are being held. It is important that this tempdir
/// always be owned, ortherwise the files will be deleted!
fn host_frontend(server: &mut Nickel, addr: &str) -> TempDir {
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
    app_js.read_to_string(&mut text).expect("app.js couldn't be read");
    app_js.seek(SeekFrom::Start(0)).unwrap();
    app_js.set_len(0).unwrap(); // delete what is there
    // the elm app uses a certain address by default, replace it
    app_js.write_all(text.replace("localhost:3733", addr).as_bytes()).unwrap();
    app_js.flush().unwrap();

    server.utilize(StaticFilesHandler::new(&dir));
    println!("hosting web ui at {}", addr);
    tmp_dir
}

/// start the json-rpc API server
#[allow(unused_variables)] // need to hold ownership of tmp_dir
pub fn start_api(project: Project, addr: &str, edit: bool) {
    // store artifacts and files into global mutex

    {
        let artifacts: Vec<ArtifactData> = project.artifacts
            .iter()
            .map(|(name, model)| model.to_data(name))
            .collect();
        let mut locked = ARTIFACTS.lock().unwrap();
        let global: &mut Vec<ArtifactData> = locked.deref_mut();
        let compare_by = |a: &ArtifactData| a.name.replace(" ", "").to_ascii_uppercase();
        global.sort_by(|a, b| compare_by(a).cmp(&compare_by(b)));
        *global = artifacts;
    }
    {
        let mut locked = PROJECT.lock().unwrap();
        let global = locked.deref_mut();
        *global = project;
    }

    let endpoint = "/json-rpc";
    let mut server = Nickel::new();

    server.get(endpoint, handle_artifacts);
    server.put(endpoint, handle_artifacts);
    server.options(endpoint,
                   middleware! { |_, mut res|
        setup_headers(&mut res);
        res.set(StatusCode::Ok);
        "ok"
    });

    // host the frontend files using a static file handler
    // and own the tmpdir for as long as needed
    let tmp_dir = host_frontend(&mut server, addr);

    server.listen(addr).expect("cannot connect to port");
}
