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
use reqwest::header::*;
use expect_macro::*;

// fn run_interop_with_server<P: AsRef<Path>>(test_base: P) {
//     let port = {
//         let mut locked = AVAILABLE_PORTS.lock();
//         let locked = expect!(locked.as_mut());
//         locked.take()
//     };
//
//     let test_base = expect!(PathDir::new(test_base));
//
//     let mut server = expect!(
//         Command::new("cargo")
//         .arg("run")
//         .arg("---")
//         .arg("serve")
//         .arg(&port.to_string())
//         // TODO: better handling of weird paths?
//         .args(&["--work-dir", &test_base.to_string_lossy()])
//         .arg("-vvv")
//         .stdout(Stdio::piped())
//         .spawn()
//     );
//
//     {
//         let rawout = expect!(server.stdout.as_mut());
//         let stdout = BufReader::new(rawout);
//         for line in stdout.lines() {
//             let line = expect!(line);
//             println!("port[{}]: {}", port, line);
//             let exp = format!("Listening on http://127.0.0.1:{}", port);
//             if line.contains(&exp) {
//                 break;
//             }
//         }
//     }
//
//     let result = panic::catch_unwind(|| {
//         let client = reqwest::Client::new();
//         let req = jrpc::Request::new(jrpc::Id::from("1"), "ReadProject");
//         let url = format!("http://127.0.0.1:{}/json-rpc", port);
//         let mut res = client
//             .post(&url)
//             .json(&req)
//             .send()
//             .expect("client not even online");
//         assert!(res.status().is_success(), "client not working");
//
//         let state = Arc::new(State { port });
//         run_interop_tests(test_base, state);
//     });
//
//     {
//         let mut locked = AVAILABLE_PORTS.lock();
//         let locked = expect!(locked.as_mut());
//         locked.give(port);
//     }
//     server.kill().expect("server didn't die");
//
//     if let Err(err) = result {
//         panic::resume_unwind(err);
//     }
// }

// fn run_interop_tests<P: AsRef<Path>>(test_base: P, state: Arc<State>) {
//     run_generic_interop_tests(
//         test_base,
//         state,
//         read_project_shim,
//         modify_project_shim,
//         assert_stuff_serve,
//     );
// }

lazy_static! {
    static ref AVAILABLE_PORTS: Mutex<UsePort> = Mutex::new(UsePort {
        available: (8500..9000).collect(),
    });
}

struct UsePort {
    available: Vec<u32>,
}

impl UsePort {
    fn take(&mut self) -> u32 {
        self.available.pop().expect("ports ran out")
    }

    fn give(&mut self, port: u32) {
        self.available.push(port)
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
    let req = jrpc::Request::new(jrpc::Id::from("1"), "ReadProject");
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
) -> result::Result<(lint::Categorized, Project), artifact_data::ModifyError> {
    println!("Modifying project via server");
    let client = reqwest::Client::new();
    let req = jrpc::Request::with_params(jrpc::Id::from("1"), "ModifyProject", operations);
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

// #[test]
// fn sanity_server() {
//     run_interop_with_server("foo");
// }

#[test]
/// #TST-read-artifact.design_only
fn interop_design_only() {
    // run_interop_with_server(INTEROP_TESTS_PATH.join("design_only"));
}
