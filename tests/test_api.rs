extern crate reqwest;
extern crate artifact_lib;
extern crate artifact_app;
extern crate ergo;
#[macro_use]
extern crate pretty_assertions;
extern crate jrpc;
use ergo::*;
use reqwest::header::*;


#[test]
fn test_basic() {
    let client = reqwest::Client::new();
    let text = reqwest::get("https://www.rust-lang.org").unwrap()
        .text().unwrap();

    let req = jrpc::Request::new(jrpc::Id::from("1"), "ReadProject");

    let mut res = client.post("http://127.0.0.1:5373/json-rpc")
        .json(&req)
        .send()
        .unwrap();
    println!("RESPONSE:\n{}", res.text().unwrap());
}
