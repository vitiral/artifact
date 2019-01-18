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
//! Handle RPC Requests
use ergo::*;
use nickel::status::StatusCode;
use nickel::{
    HttpRouter, MediaType, MiddlewareResult, Nickel, Request, Response, StaticFilesHandler,
};
use tar::Archive;
use tempdir::TempDir;
// use jsonrpc_core::{Error as RpcError, ErrorCode, IoHandler, Params, RpcMethodSync};
use jrpc;
use std::mem;
use std::result;

use frontend;
use serve;

use artifact_data::*;
use dev_prelude::*;

// ----- SERVER -----

pub fn start_api(cmd: super::Serve) {
    let endpoint = "/json-rpc";
    let mut server = Box::new(Nickel::new());

    server.post(endpoint, handle_rpc);
    server.get(endpoint, handle_rpc);
    server.put(endpoint, handle_rpc);
    server.options(endpoint, handle_options);

    let running = Arc::new(AtomicBool::new(true));
    let r = running.clone();
    ctrlc::set_handler(move || {
        r.store(false, AtomicOrdering::SeqCst);
    })
    .expect("Error setting Ctrl-C handler");

    // host the frontend files using a static file handler
    // and own the tmpdir for as long as needed
    let _tmp_dir = host_frontend(&mut server, &cmd);

    // everything in a thread has to be owned by the thread
    let addr = format!("127.0.0.1:{}", cmd.port);
    let _th = spawn(move || {
        server.listen(addr).expect("cannot connect to port");
    });

    println!("Exit with ctrlc+C or SIGINT");
    while running.load(AtomicOrdering::SeqCst) {
        sleep(Duration::new(0, 10 * 1e6 as u32));
    }

    debug!("Got SIGINT, cleaning up");
    let locked = super::LOCKED.lock().unwrap();
    mem::forget(locked); // never unlock again
    debug!("All cleaned up, exiting");
}

// ----- API CALLS -----

fn rpc_read_project(id: jrpc::Id, params: Option<json::Value>) -> jrpc::Response<json::Value> {
    info!("ReadProject");
    let mut locked = super::LOCKED.lock().unwrap();
    let locked = locked.as_mut().unwrap();

    if let Some(params) = params {
        if let Err(res) = handle_read_project_params(&id, locked, params) {
            return res;
        }
    }

    jrpc::Response::success(id, json::to_value(locked).expect("serde"))
}

fn handle_read_project_params(
    id: &jrpc::Id,
    project: &mut ProjectResult,
    params: json::Value,
) -> ::std::result::Result<(), jrpc::Response<json::Value>> {
    let params: ParamsReadProject = match json::from_value(params) {
        Ok(o) => o,
        Err(err) => {
            return Err(jrpc::Response::error(
                id.clone(),
                jrpc::ErrorCode::InvalidParams,
                err.to_string(),
                None,
            ));
        }
    };

    if params.reload {
        let (lints, new_project) = match read_project(&project.project.settings.base) {
            Ok(v) => v,
            Err(err) => {
                return Err(jrpc::Response::error(
                    id.clone(),
                    jrpc::ErrorCode::ServerError(-32000),
                    ModifyErrorKind::InvalidFromLoad.to_string(),
                    Some(json::to_value(&err).unwrap()),
                ));
            }
        };
        *project = ProjectResult {
            project: new_project,
            lints: lints,
        };
    }

    Ok(())
}

fn rpc_modify_project(id: jrpc::Id, params: Option<json::Value>) -> jrpc::Response<json::Value> {
    info!("ModifyProject");
    let mut locked = super::LOCKED.lock().unwrap();
    let locked = locked.as_mut().unwrap();

    let params = match params {
        Some(p) => p,
        None => {
            return jrpc::Response::error(
                id,
                jrpc::ErrorCode::InvalidParams,
                "No 'params'".to_string(),
                None,
            );
        }
    };

    let ops: Vec<ArtifactOp> = match json::from_value(params) {
        Ok(o) => o,
        Err(err) => {
            return jrpc::Response::error(id, jrpc::ErrorCode::InvalidParams, err.to_string(), None);
        }
    };

    let (lints, project) = match modify_project(&locked.project.settings.base, ops) {
        Ok(r) => r,
        Err(err) => {
            return jrpc::Response::error(
                id,
                jrpc::ErrorCode::ServerError(-32000),
                err.kind.to_string(),
                Some(json::to_value(&err.lints).unwrap()),
            );
        }
    };

    let result = ProjectResult { project, lints };
    *locked = result;
    let value = json::to_value(locked).expect("serde");

    jrpc::Response::success(id, value)
}

// ----- HANDLE ENDPOINTS -----

/// Handle the `/artifacts` endpoint.
fn handle_rpc<'a>(req: &mut Request, mut res: Response<'a>) -> MiddlewareResult<'a> {
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

    debug!("request: {}", body);
    let request: result::Result<jrpc::Request<Method, json::Value>, jrpc::Error<json::Value>> =
        jrpc::parse_request(body);

    let request = match request {
        Ok(r) => r,
        Err(err) => return res.send(json::to_string(&err).unwrap()),
    };

    let id = request.id.to_id().clone().unwrap_or(jrpc::Id::Null);

    let response = match request.method {
        Method::ReadProject => rpc_read_project(id, request.params),
        Method::ModifyProject => rpc_modify_project(id, request.params),
    };
    let out = res.send(json::to_string(&response).unwrap());
    debug!("Exiting handle_rpc");
    out
}

/// Host the frontend web-server on `/`, returning the tempdir where the
/// static files are being held. It is important that this tempdir
/// always be owned, ortherwise the files will be deleted!
fn host_frontend(server: &mut Nickel, cmd: &serve::Serve) -> TempDir {
    let init = ProjectInitialSer {
        project: None,
        web_type: WebType::Editable,
    };
    let tmp_dir = expect!(TempDir::new("artifact-web-ui"));
    expect!(frontend::unpack_frontend(&tmp_dir, &init));

    server.utilize(StaticFilesHandler::new(&tmp_dir.path()));
    println!("Hosting frontend at {}", cmd.port);
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
