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
#[macro_use]
extern crate artifact_test;
extern crate assert_cli;
#[macro_use]
extern crate expect_macro;
extern crate jrpc;
#[macro_use]
extern crate pretty_assertions;
extern crate reqwest;

use std::panic;
use std::process::{Command, Stdio};
use std::result;

use artifact_test::*;
use expect_macro::*;
use reqwest::header::*;

fn run_interop_tests<P: AsRef<Path>>(test_base: P) {
    run_generic_interop_tests(test_base, run_server_test);
}

fn run_server_test(project_path: PathDir) {
    let port = AVAILABLE_PORTS.take();

    let args = &[
        "run",
        "--",
        "serve",
        &port.to_string(),
        // TODO: better handling of weird paths?
        "--work-dir",
        &project_path.to_string_lossy(),
        "-vvv",
    ];

    println!("Running: cargo with {:?}", args);
    let mut server = expect!(
        Command::new("cargo")
            .args(args)
            .stdout(Stdio::piped())
            .spawn()
    );

    {
        let rawout = expect!(server.stdout.as_mut());
        let stdout = BufReader::new(rawout);
        for line in stdout.lines() {
            let line = expect!(line);
            println!("port[{}]: {}", port, line);
            let exp = format!("Listening on http://127.0.0.1:{}", port);
            if line.contains(&exp) {
                break;
            }
        }
    }

    let result = panic::catch_unwind(|| {
        let client = reqwest::Client::new();
        let req = jrpc::Request::new(jrpc::Id::from("1"), Method::ReadProject);
        let url = format!("http://127.0.0.1:{}/json-rpc", port);
        let mut res = expect!(
            client.post(&url).json(&req).send(),
            "client not even online"
        );
        assert!(res.status().is_success(), "client not working");

        let state = Arc::new(State { port });
        run_generic_interop_test(
            project_path.clone(),
            state,
            read_project_shim,
            modify_project_shim,
            assert_stuff_serve,
        );
    });

    expect!(server.kill(), "server didn't die");
    AVAILABLE_PORTS.give(port);

    if let Err(err) = result {
        panic::resume_unwind(err);
    }
}

#[derive(Debug)]
struct State {
    port: u32,
}

fn read_project_shim(
    project_path: PathDir,
    state: Arc<State>,
) -> result::Result<(lint::Categorized, Project), lint::Categorized> {
    println!("Modifying project via server");
    let client = reqwest::Client::new();
    let req = jrpc::Request::new(jrpc::Id::from("1"), Method::ReadProject);
    let url = format!("http://127.0.0.1:{}/json-rpc", state.port);
    let mut res = client
        .post(&url)
        .json(&req)
        .send()
        .expect("client not even online");
    assert!(res.status().is_success(), "client not working");

    let response: jrpc::Response<ProjectResult> = res.json().expect("invalid json response");
    match response {
        jrpc::Response::Ok(v) => Ok((v.result.lints, v.result.project)),
        jrpc::Response::Err(v) => {
            let data = expect!(v.error.data);
            let lints: lint::Categorized = expect!(json::from_value(data));
            Err(lints)
        }
    }
}

/// Simply calls `artifact_data::modify_project(project_path, operations)`
///
/// Used to satisfy the type requirements of `Fn` (cannot accept `AsRef`)
fn modify_project_shim(
    project_path: PathDir,
    operations: Vec<ArtifactOp>,
    state: Arc<State>,
) -> result::Result<(lint::Categorized, Project), ModifyError> {
    println!("Modifying project via server");
    let client = reqwest::Client::new();
    let req = jrpc::Request::with_params(jrpc::Id::from("1"), Method::ModifyProject, operations);
    let url = format!("http://127.0.0.1:{}/json-rpc", state.port);
    let mut res = client
        .post(&url)
        .json(&req)
        .send()
        .expect("client not even online");
    assert!(res.status().is_success(), "client not working");

    let response: jrpc::Response<ProjectResult> = res.json().expect("invalid json response");
    match response {
        jrpc::Response::Ok(v) => Ok((v.result.lints, v.result.project)),
        jrpc::Response::Err(v) => {
            println!("Got error: {:#?}", v);
            let data = expect!(v.error.data);
            let lints: lint::Categorized = expect!(json::from_value(data));
            let kind = expect!(ModifyErrorKind::from_str(&v.error.message));
            Err(ModifyError { lints, kind })
        }
    }
}

/// Just a shim for now to ignore the state.
fn assert_stuff_serve(
    project_path: PathDir,
    _state: Arc<State>,
    load_lints: Categorized,
    project: Option<Project>,
    expect: ExpectStuff,
) {
    assert_stuff_data(project_path, (), load_lints, project, expect)
}

#[test]
fn serve_interop_source_only() {
    run_interop_tests(INTEROP_TESTS_PATH.join("source_only"));
}

// TODO(maybe): the server doesn't start because there is a critical
//   error.
// #[test]
// fn serve_interop_source_invalid() {
//     run_interop_tests(INTEROP_TESTS_PATH.join("source_invalid"));
// }

#[test]
fn serve_interop_project_empty() {
    run_interop_tests(INTEROP_TESTS_PATH.join("empty"));
}

#[test]
fn serve_interop_design_only() {
    run_interop_tests(INTEROP_TESTS_PATH.join("design_only"));
}

#[test]
fn serve_interop_basic() {
    run_interop_tests(INTEROP_TESTS_PATH.join("basic"));
}

#[test]
fn serve_interop_lints() {
    run_interop_tests(INTEROP_TESTS_PATH.join("lints"));
}
