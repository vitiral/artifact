use std::fs;
use std::path;
use std::ascii::AsciiExt;
use std::io::{Read, Write, Seek, SeekFrom};
use std::str;
use std::sync::Mutex;

//extern crate rustc_serialize;
//#[macro_use] extern crate nickel;
//#[macro_use] extern crate lazy_static;
//extern crate jsonrpc_core;

//#[macro_use] extern crate serde;
//#[macro_use] extern crate serde_derive;
//extern crate serde_json;

use nickel::{
    Request, Response, MiddlewareResult,
    Nickel, HttpRouter, MediaType,
    StaticFilesHandler};
use nickel::status::StatusCode;
use tar::Archive;
use tempdir::TempDir;

use core::{ArtifactData, LocData};

mod handler;

const WEB_FRONTEND_TAR: &'static [u8] = include_bytes!("web-ui.tar");

lazy_static! {
    //#[derive(RustcDecodable, RustcEncodable, Serialize, Deserialize, Debug)]
    static ref ARTIFACTS: Mutex<Vec<ArtifactData>> = Mutex::new(vec![
        ArtifactData {
            id: 1,
            name: "REQ-foo".to_string(),
            path: "path/to/definition".to_string(),
            text: "text\nnew line".to_string(),
            partof: vec!["REQ-partof1".to_string(), "REQ-partof2".to_string()],
            parts: vec!["REQ-bar".to_string(), "REQ-parts2".to_string()],
            loc: None,
            completed: 0.0,
            tested: 0.0,
        },
        ArtifactData {
            id: 2,
            name: "REQ-bar".to_string(),
            path: "path/to/definition".to_string(),
            text: "text\nnew line".to_string(),
            partof: vec!["REQ-foo".to_string(), "REQ-partof2".to_string()],
            parts: vec!["REQ-parts1".to_string(), "REQ-parts2".to_string()],
            loc: Some( LocData{path: "path/implemented/at".to_string(), row: 42, col: 66}),
            completed: 0.0,
            tested: 0.0,
        },
    ]);
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

fn handle_artifacts<'a> (req: &mut Request, mut res: Response<'a>) 
        -> MiddlewareResult<'a> 
{
    setup_headers(&mut res);
    debug!("handling json-rpc request");

    let mut body = vec![];
    req.origin.read_to_end(&mut body).unwrap();
    let body = match str::from_utf8(&body) {
        Ok(b) => b,
        Err(e) => {
            res.set(StatusCode::BadRequest);
            return res.send(format!("invalid utf8: {:?}", e));
        },
    };

    trace!("request: {:?}", body);
    match handler::RPC_HANDLER.handle_request_sync(body) {
        Some(body) => {
            trace!("- response {}", body);
            config_json_res(&mut res);
            res.send(body)
        },
        None => {
            let msg = "InternalServerError: Got None from json-rpc handler";
            error!("{}", msg);
            res.set(StatusCode::InternalServerError);
            res.send(msg)
        }
    }
}

fn unpack_app(dir: &path::Path, addr: &str) {
    info!("unpacking web-ui at: {}", dir.display());
    let mut archive = Archive::new(WEB_FRONTEND_TAR);
    archive.unpack(&dir).expect("unable to unpack web frontend");

    // replace the default ip address with the real one
    let app_js_path = dir.join("app.js");
    let mut app_js = fs::OpenOptions::new()
        .read(true)
        .write(true)
        .open(app_js_path).expect("couldn't open app.js");
    let mut text = String::new();
    app_js.read_to_string(&mut text).expect("app.js couldn't be read");
    app_js.seek(SeekFrom::Start(0)).unwrap();    
    app_js.set_len(0).unwrap(); // delete what is there
    // the elm app uses a certain address by default, replace it
    //app_js.write_all(text.replace("http://localhost:3733", addr).as_bytes()).unwrap();
    app_js.write_all(text.replace("localhost:3733", addr).as_bytes()).unwrap();
    app_js.flush().unwrap();
}

pub fn start_api(artifacts: Vec<ArtifactData>, addr: &str) {
    {
        let mut locked = ARTIFACTS.lock().unwrap();
        let global: &mut Vec<ArtifactData> = locked.as_mut();
        let compare_by = |a: &ArtifactData| a.name.replace(" ", "").to_ascii_uppercase();
        global.sort_by(|a, b| compare_by(a).cmp(&compare_by(b)));
        *global = artifacts;
    }
    // it is important that tmp_dir never goes out of scope
    // or the webapp will be deleted!
    let tmp_dir = TempDir::new("rst-web-ui")
        .expect("unable to create temporary directory");
    let app_dir = tmp_dir.path();
    debug!("unpacking webapp in {}", app_dir.display());
    unpack_app(app_dir, addr);

    let endpoint = "/json-rpc";
    let mut server = Nickel::new();

    server.get(endpoint, handle_artifacts);
    server.put(endpoint, handle_artifacts);
    server.options(endpoint, middleware! { |_, mut res|
        setup_headers(&mut res);
        res.set(StatusCode::Ok);
        "ok"
    });

    //server.utilize(StaticFilesHandler::new("web-ui/dist"));
    server.utilize(StaticFilesHandler::new(&tmp_dir));

    server.listen(addr).expect("cannot connect to port");
}
