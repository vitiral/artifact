use dev_prelude::*;
use http;
use jrpc;

macro_rules! create_fetch_task {
    [$ctx:ident, $jreq:ident] => {{
        let callback = $ctx.send_back(handle_response);
        let request = http::Request::post("/json-rpc")
            .body(json::to_string(&$jreq).expect("request-ser"))
            .expect("create request");
        FetchTask::new(request, callback)
    }}
}

/// Send a request to fetch the project.
pub(crate) fn handle_fetch_project(
    model: &mut Model,
    context: &mut Env<Context, Model>,
    reload: bool,
) -> bool {
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
pub(crate) fn handle_send_update(
    model: &mut Model,
    context: &mut Env<Context, Model>,
    ids: Vec<usize>,
) -> bool {
    if model.fetch_task.is_some() {
        push_logs_fetch_in_progress(model);
        return false;
    }

    let params: Vec<_> = ids.iter()
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

fn new_rpc_id() -> jrpc::Id {
    jrpc::Id::Int(new_id() as i64)
}

/// Handle response of fetch
fn handle_response(response: http::Response<String>) -> Msg {
    let status = response.status();
    if !status.is_success() {
        let html = format!(
            "<div>Received {} from server: {}</div>",
            status,
            response.into_body(),
        );

        return Msg::RecvError(vec![Log::error(html)]);
    }

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
