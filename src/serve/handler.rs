//! Handle RPC Requests
use nickel::{HttpRouter, MediaType, MiddlewareResult, Nickel, Request, Response,
             StaticFilesHandler};
use nickel::status::StatusCode;
use ergo::json;
use tar::Archive;
use tempdir::TempDir;
use jsonrpc_core::{IoHandler, Error as RpcError, ErrorCode, Params, RpcMethodSync};
use std::result;
use std::mem;

// use api::crud;
use serve;

use dev_prelude::*;
use artifact_data::*;

lazy_static! {
    pub static ref RPC_HANDLER: IoHandler = init_rpc_handler();
}

const WEB_FRONTEND_TAR: &'static [u8] = include_bytes!("../../web-ui/target/web-ui.tar");
const REPLACE_FLAGS: &str = "{/* REPLACE WITH FLAGS */}";


#[derive(Debug, Serialize, Deserialize)]
struct Flags {
    readonly: bool,
    /// TODO: rename src_url
    path_url: String,
}

// ----- SERVER -----

pub fn start_api(cmd: super::Serve) {
    let endpoint = "/json-rpc";
    let mut server = Box::new(Nickel::new());

    server.get(endpoint, handle_artifacts);
    server.put(endpoint, handle_artifacts);
    server.options(endpoint, handle_options);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || { r.store(false, AtomicOrdering::SeqCst); })
        .expect("Error setting Ctrl-C handler");

    // host the frontend files using a static file handler
    // and own the tmpdir for as long as needed
    let tmp_dir = host_frontend(&mut server, &cmd);

    // everything in a thread has to be owned by the thread
    let addr = format!("127.0.0.1:{}", cmd.port);
    let th = spawn(move || {
        server.listen(addr).expect("cannot connect to port");
    });

     println!("exit with ctrlc+C or SIGINT");
     while running.load(AtomicOrdering::SeqCst) {
         sleep(Duration::new(0, 10 * 1e6 as u32));
     }

    debug!("Got SIGINT, cleaning up");
    let locked = super::LOCKED.lock().unwrap();
    mem::forget(locked); // never unlock again
    debug!("All cleaned up, exiting");
}

// ----- API CALLS -----

/// the rpc initializer that implements the API spec
fn init_rpc_handler() -> IoHandler {
    let mut handler = IoHandler::new();
    // FIXME
    // handler.add_method("CreateArtifacts", crud::CreateArtifacts);

    // TODO: rename to ReadProject instead of ReadArtifacts
    handler.add_method("ReadArtifacts", ReadArtifacts);

    // handler.add_method("UpdateArtifacts", crud::UpdateArtifacts);
    // handler.add_method("DeleteArtifacts", crud::DeleteArtifacts);
    handler
}

/// `ReadArtifacts` API Handler
pub struct ReadArtifacts;
impl RpcMethodSync for ReadArtifacts {
    fn call(&self, _: Params) -> result::Result<json::Value, RpcError> {
        info!("ReadArtifacts");
        let locked = super::LOCKED.lock().unwrap();
        let locked = locked.as_ref().unwrap();
        Ok(json::to_value(&locked.project).expect("serde"))
    }
}

// ----- HANDLE ENDPOINTS -----

/// Handle the `/artifacts` endpoint.
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

    debug!("request: {:?}", body);
    match RPC_HANDLER.handle_request_sync(body) {
        Some(body) => {
            config_json_res(&mut res);
            trace!("- response {}", body);
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

/// Host the frontend web-server on `/`, returning the tempdir where the
/// static files are being held. It is important that this tempdir
/// always be owned, ortherwise the files will be deleted!
fn host_frontend(server: &mut Nickel, cmd: &serve::Serve) -> TempDir {
    // it is important that tmp_dir never goes out of scope
    // or the webapp will be deleted!
    let tmp_dir = TempDir::new("artifact-web-ui").expect("unable to create temporary directory");
    let dir = tmp_dir.path().to_path_buf(); // we have to clone this because *borrow*
    info!("Unpacking web-ui at: {}", dir.display());

    let mut archive = Archive::new(WEB_FRONTEND_TAR);
    archive.unpack(&dir).expect("Unable to unpack web frontend");

    // replace the default ip address with the real one
    let app_js_path = dir.join("app.js");
    let mut app_js = FileEdit::edit(app_js_path).unwrap();
    let mut text = String::new();
    app_js
        .read_to_string(&mut text)
        .expect("app.js couldn't be read");
    app_js.seek(SeekFrom::Start(0)).unwrap();
    app_js.set_len(0).unwrap(); // delete what is there
    // the elm app uses a certain address by default, replace it

    assert!(text.contains(REPLACE_FLAGS));
    let flags = Flags {
        readonly: true,
        path_url: "".into(),
    };
    app_js
        .write_all(
            text.replace(REPLACE_FLAGS, &json::to_string(&flags).unwrap())
                .as_bytes(),
        )
        .unwrap();
    app_js.flush().unwrap();

    server.utilize(StaticFilesHandler::new(&dir));
    println!("Hosting web ui at {}", cmd.port);
    tmp_dir
}

// ----- HEADER HELPERS -----

/// Setup the response headers
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

/// Config response for returning JSON
fn config_json_res(res: &mut Response) {
    res.set(MediaType::Json);
    res.set(StatusCode::Ok);
}

fn handle_options<'a>(_: &mut Request, mut res: Response<'a>) -> MiddlewareResult<'a> {
    setup_headers(&mut res);
    res.set(StatusCode::Ok);
    res.send("ok")
}

