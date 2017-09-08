use dev_prefix::*;

use std::time::Duration;
use std::mem;
use std::sync::Mutex;
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;

use nickel::{HttpRouter, MediaType, MiddlewareResult, Nickel, Request, Response,
             StaticFilesHandler};
use nickel::status::StatusCode;
use serde_json;

use tar::Archive;
use tempdir::TempDir;

use types::{Project, ServeCmd};
use export::ProjectData;
use ctrlc;

mod constants;
pub mod utils;
mod handler;
mod crud;

#[cfg(test)]
mod tests;

const WEB_FRONTEND_TAR: &'static [u8] = include_bytes!("data/web-ui.tar");
const REPLACE_FLAGS: &str = "{/* REPLACE WITH FLAGS */}";

pub struct LockedData {
    pub cmd: ServeCmd,
    pub project: Project,
    pub project_data: ProjectData,
}

lazy_static! {
    static ref LOCKED: Mutex<LockedData> = Mutex::new(
        LockedData {
            cmd: ServeCmd::default(),
            project_data: ProjectData::default(),
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
    info!("Unpacking web-ui at: {}", dir.display());

    let mut archive = Archive::new(WEB_FRONTEND_TAR);
    archive.unpack(&dir).expect("Unable to unpack web frontend");

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
    println!("Hosting web ui at {}", cmd.addr);
    tmp_dir
}

/// start the json-rpc API server
#[allow(unused_variables)] // need to hold ownership of tmp_dir
pub fn start_api(project: Project, cmd: &ServeCmd) {
    // store artifacts and files into global mutex

    {
        let mut locked = LOCKED.lock().unwrap();
        let lref = locked.deref_mut();
        lref.project_data = project.to_data();
        lref.project = project;
        lref.cmd = cmd.clone();
    }

    let endpoint = "/json-rpc";
    let mut server = Box::new(Nickel::new());

    server.get(endpoint, handle_artifacts);
    server.put(endpoint, handle_artifacts);
    server.options(endpoint, handle_options);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || { r.store(false, Ordering::SeqCst); })
        .expect("Error setting Ctrl-C handler");

    // host the frontend files using a static file handler
    // and own the tmpdir for as long as needed
    let tmp_dir = host_frontend(&mut server, cmd);

    // everything in a thread has to be owned by the thread
    let addr = cmd.addr.clone();
    let th = thread::spawn(move || {
        server.listen(&addr).expect("cannot connect to port");
    });

    println!("exit with ctrlc+C or SIGINT");
    while running.load(Ordering::SeqCst) {
        thread::sleep(Duration::new(0, 10 * 1e6 as u32));
    }

    debug!("Got SIGINT, cleaning up");
    let locked = LOCKED.lock().unwrap();
    mem::forget(locked); // never unlock again
    debug!("All cleaned up, exiting");
}
