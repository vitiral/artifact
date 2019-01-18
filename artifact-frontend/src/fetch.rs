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
use crate::dev_prelude::*;
use http;
use jrpc;
use stdweb::Value;

macro_rules! create_fetch_task {
    [$ctx:ident, $jreq:ident] => {{
        let callback = $ctx.send_back(handle_response);
        let body = json::to_string(&$jreq).expect("request-ser");
        let request = http::Request::post("/json-rpc")
            .body(Value::String(body))
            .expect("create request");
        FetchTask::new(request, callback)
    }}
}

pub(crate) fn start_fetch_initial(model: &mut Model, context: &mut Env<Context, Model>) -> bool {
    if model.fetch_task.is_some() {
        panic!("This should only be called first.")
    }

    let callback = context.send_back(handle_response_initial);
    let request = http::Request::get("initial.json")
        .body(Value::Null)
        .expect("initial request");
    model.fetch_task = Some(FetchTask::new(request, callback));
    false
}

/// Handle response of fetch
fn handle_response_initial(response: http::Response<String>) -> Msg {
    let response = match handle_status(response) {
        Ok(r) => r,
        Err(msg) => return msg,
    };

    let body = response.into_body();
    let init: ProjectInitialSer = expect!(json::from_str(&body), "response-serde");

    Msg::RecvInitial(init)
}

/// Send a request to fetch the project.
pub(crate) fn start_fetch_project(
    model: &mut Model,
    context: &mut Env<Context, Model>,
    reload: bool,
) -> bool {
    if model.web_type == WebType::Static {
        push_logs_fetch_invalid(model);
        return false;
    }

    if model.fetch_task.is_some() {
        push_logs_fetch_in_progress(model);
        false
    } else {
        let request = jrpc::Request::with_params(
            new_rpc_id(),
            Method::ReadProject,
            ParamsReadProject { reload: reload },
        );
        model.fetch_task = Some(create_fetch_task!(context, request));
        false
    }
}

/// Send a request to alter/update the project and get the results.
pub(crate) fn start_send_update(
    model: &mut Model,
    context: &mut Env<Context, Model>,
    ids: Vec<usize>,
) -> bool {
    if model.fetch_task.is_some() {
        push_logs_fetch_in_progress(model);
        return false;
    }

    let params: Vec<_> = ids
        .iter()
        .map(|edit_id| {
            let edit = expect!(model.editing.get(edit_id), "FIXME: log msg if dne");
            if let Some(orig_id) = edit.original_id {
                ArtifactOpSer::Update {
                    artifact: edit.to_im(),
                    orig_id,
                }
            } else {
                ArtifactOpSer::Create {
                    artifact: edit.to_im(),
                }
            }
        })
        .collect();

    let jid = new_rpc_id();
    let request = jrpc::Request::with_params(jid.clone(), Method::ModifyProject, params);
    model.fetch_task = Some(create_fetch_task!(context, request));
    model.updating.insert(jid, ids);
    true
}

/// Handle the receiving of the project.
pub(crate) fn handle_recv_project(model: &mut Model, jid: &jrpc::Id, project: Arc<ProjectSer>) {
    model.shared = project;
    model.fetch_task = None;
    if let Some(mut ids) = model.updating.remove(jid) {
        for id in ids.drain(..) {
            model.complete_editing(id);
        }
    }
}

fn push_logs_fetch_in_progress(model: &mut Model) {
    model.push_logs(vec![Log::info(
        "<div>A fetch is already in progress, try again later.</div>".to_string(),
    )]);
}

fn push_logs_fetch_invalid(model: &mut Model) {
    model.push_logs(vec![Log::error(
        "<div>Internal Error: a fetch was invalid and \
         should not have been possible</div>"
            .to_string(),
    )]);
}

fn new_rpc_id() -> jrpc::Id {
    jrpc::Id::Int(new_id() as i64)
}

/// Handle response of fetch
fn handle_response(response: http::Response<String>) -> Msg {
    let response = match handle_status(response) {
        Ok(r) => r,
        Err(msg) => return msg,
    };

    let body = response.into_body();
    let response: jrpc::Response<ProjectResultSer> =
        expect!(json::from_str(&body), "response-serde");

    let result = match response {
        jrpc::Response::Ok(r) => r,
        jrpc::Response::Err(err) => {
            return Msg::RecvError(vec![Log::error(format!(
                "<div>received jrpc Error: {:?}</div>",
                err
            ))]);
        }
    };

    Msg::RecvProject(result.id, Arc::new(result.result.project))
}

fn handle_status(response: http::Response<String>) -> Result<http::Response<String>, Msg> {
    let status = response.status();
    if !status.is_success() {
        let html = format!(
            "<div>Received {} from server: {}</div>",
            status,
            response.into_body(),
        );

        Err(Msg::RecvError(vec![Log::error(html)]))
    } else {
        Ok(response)
    }
}
