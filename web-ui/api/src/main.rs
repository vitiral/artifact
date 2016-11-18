#![feature(proc_macro)]

use std::ops::Deref;
use std::io::Read;
use std::str;
use std::sync::Mutex;

extern crate rustc_serialize;
#[macro_use] extern crate nickel;
#[macro_use] extern crate lazy_static;
extern crate jsonrpc_core;

#[macro_use] extern crate serde;
#[macro_use] extern crate serde_derive;
extern crate serde_json;

use nickel::{
    Request, Response, MiddlewareResult,
    Nickel, HttpRouter, MediaType,
    // Traits
    JsonBody};
use nickel::status::StatusCode;

mod handler;

lazy_static! {
    //#[derive(RustcDecodable, RustcEncodable, Serialize, Deserialize, Debug)]
    static ref ARTIFACTS: Mutex<Vec<Artifact>> = Mutex::new(vec![
        Artifact {
            id: 1,
            name: "Sally".to_string(),
            level: 2,
        },
        Artifact {
            id: 2,
            name: "Lance".to_string(),
            level: 1,
        },
    ]);
}

#[derive(RustcDecodable, RustcEncodable, Serialize, Deserialize, Eq, PartialEq, Debug)]
struct Artifact {
    pub id: u64,
    pub name: String,
    pub level: i64,
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
    println!("* handle json-rpc request");

    let mut body = vec![];
    req.origin.read_to_end(&mut body).unwrap();
    let body = match str::from_utf8(&body) {
        Ok(b) => b,
        Err(e) => {
            res.set(StatusCode::BadRequest);
            return res.send(format!("invalid utf8: {:?}", e));
        },
    };

    println!("- request: {:?}", body);
    match handler::RPC_HANDLER.handle_request_sync(body) {
        Some(body) => {
            println!("- response {}", body);
            config_json_res(&mut res);
            res.send(body)
        },
        None => {
            let msg = "InternalServerError: Got None from json-rpc handler";
            println!(msg);
            res.set(StatusCode::InternalServerError);
            res.send(msg)
        }
    }
}


fn main() {
    let endpoint = "/json-rpc";
    let mut server = Nickel::new();

    server.get(endpoint, handle_artifacts);
    server.put(endpoint, handle_artifacts);
    server.options(endpoint, middleware! { |_, mut res|
        setup_headers(&mut res);
        res.set(StatusCode::Ok);
        "ok"
    });

    server.listen("127.0.0.1:4000").expect("canot connect to port");
}
