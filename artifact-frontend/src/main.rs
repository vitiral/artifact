/* artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */
#![recursion_limit = "128"]
#![allow(unused_imports)]

#[macro_use]
extern crate artifact_ser;
#[macro_use]
extern crate ergo_config;
#[macro_use]
extern crate ergo_std;
#[macro_use]
extern crate expect_macro;
extern crate http;
extern crate jrpc;
#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate yew;
extern crate yew_simple;

use std::result;

use http::response::Parts;
use yew::format::{Json, Nothing};
use yew::prelude::*;
use yew::services::Task;
use yew::services::websocket::{WebSocketService, WebSocketStatus, WebSocketTask};

mod artifact;
mod dev_prelude;
mod edit;
mod example;
mod graph;
mod name;
mod nav;
mod view;

use dev_prelude::*;

lazy_static! {
    static ref NAME_URL: Regex =
        Regex::new(&format!(r"(?i)(?:artifacts/)?({})", NAME_VALID_STR)).expect("regex");
    static ref EDIT_URL: Regex = Regex::new(r"(?i)edit/(\d+)").expect("regex");
    static ref ATOMIC_ID: AtomicUsize = ATOMIC_USIZE_INIT;
}

pub(crate) fn router(info: yew_simple::RouteInfo) -> Msg {
    let view = get_view(info.url.fragment().unwrap_or_default());
    Msg::SetView(view)
}

fn get_view(hash: &str) -> View {
    if hash.to_ascii_lowercase() == "graph" || hash == "" {
        View::Graph
    } else if let Some(cap) = NAME_URL.captures(hash) {
        let name = name!(&cap[1]);
        View::Artifact(name)
    } else if let Some(cap) = EDIT_URL.captures(hash) {
        let id = match usize::from_str(&cap[1]) {
            Ok(id) => id,
            Err(_) => return View::NotFound,
        };
        View::Edit(id)
    } else {
        View::NotFound
    }
}

impl Component<Context> for Model {
    type Msg = Msg;
    type Properties = ();

    fn create(_: Self::Properties, context: &mut Env<Context, Self>) -> Self {
        let project: ProjectSer = yaml::from_str(example::YAML).unwrap();
        let router = yew_simple::RouterTask::new(context, &router);
        let url = router.current_url();

        let mut out = Model {
            shared: Arc::new(project),
            view: get_view(&url.fragment().unwrap_or_default()),
            router: Arc::new(router),
            nav: Nav::default(),
            graph: Graph::default(),
            fetch_task: None,
            console: Arc::new(ConsoleService::new()),
            logs: Logs::default(),
            window: ::stdweb::web::window(),
            editing: IndexMap::new(),
        };
        out.nav.search.on = true;
        out.nav.editing.on = true;
        out
    }

    fn update(&mut self, msg: Self::Msg, context: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Msg::Batch(mut batch) => {
                let count: usize = batch
                    .drain(..)
                    .map(|msg| update_model(self, msg, context) as usize)
                    .sum();
                count != 0
            }
            msg @ _ => update_model(self, msg, context),
        }
    }
}

fn update_model(model: &mut Model, msg: Msg, context: &mut Env<Context, Model>) -> ShouldRender {
    match msg {
        Msg::SetView(view) => model.view = view,
        Msg::Ignore => return false,

        Msg::ToggleSearch => model.nav.search.on = !model.nav.search.on,
        Msg::ToggleEditing => model.nav.editing.on = !model.nav.editing.on,
        Msg::SetNavSearch(v) => model.nav.search.value = v,
        Msg::SetNavEditing(v) => model.nav.editing.value = v,

        Msg::SetGraphSearch(v) => model.graph.search = v,

        Msg::FetchProject => {
            if model.fetch_task.is_some() {
                return false;
            }
            let callback = context.send_back(fetch_fn);
            let request = jrpc::Request::new(jrpc::Id::Int(1), Method::ReadProject);
            let request = http::Request::post("/json-rpc")
                .body(json::to_string(&request).expect("request-ser"))
                .expect("create request");
            model.fetch_task = Some(FetchTask::new(request, callback));
        }
        Msg::RecvProject(project) => {
            model.shared = Arc::new(project);
        }
        Msg::SendUpdate(id) => unimplemented!("FIXME"),

        Msg::PushLogs(logs) => push_logs(model, logs),
        Msg::ClearLogs(clear) => clear_logs(model, clear),

        Msg::EditArtifact(id, field) => edit_artifact(model, id, field),
        Msg::StartEdit(id, ty) => {
            let artifact = match ty {
                StartEditType::New => ArtifactEdit::default(),
                StartEditType::Current => {
                    if let View::Artifact(ref name) = model.view {
                        let art = expect!(
                            model.shared.artifacts.get(name),
                            "{} viewed but not here",
                            name,
                        );
                        ArtifactEdit::from_artifact(art)
                    } else {
                        panic!("wrong view");
                    }
                }
            };
            model.editing.insert(id, artifact);
        }
        Msg::Batch(_) => panic!("batch within a batch"),
    }
    true
}

fn edit_artifact(model: &mut Model, id: usize, field: Field) {
    let artifact = match model.editing.get_mut(&id) {
        Some(a) => a,
        None => panic!("TODO: got invalid editing artifact"),
    };
    match field {
        Field::Name(v) => artifact.name = v,
        Field::File(v) => artifact.file = v,
        Field::Done(v) => artifact.done = v,
        Field::Text(v) => artifact.text = v,
        Field::Partof(index, op) => match op {
            FieldOp::Create => artifact.partof.push("".into()),
            FieldOp::Update(v) => artifact.partof[index] = v,
            FieldOp::Delete => {
                artifact.partof.remove(index);
            }
        },
    }
}

fn push_logs(model: &mut Model, mut logs: Vec<Log>) {
    for log in logs.drain(..) {
        let id = new_id();
        match log.level {
            LogLevel::Error => {
                model.logs.error.insert(id, log);
            }
            LogLevel::Info => {
                // FIXME: add scheduling to remove id for info
                model.logs.error.insert(id, log);
            }
        }
    }
}

fn clear_logs(model: &mut Model, mut clear: ClearLogs) {
    match clear {
        ClearLogs::Error(mut ids) => for id in ids.drain(..) {
            model.logs.error.remove(&id);
        },
        ClearLogs::ErrorAll => {
            model.logs.error.clear();
        }
    }
}

fn fetch_fn(response: http::Response<String>) -> Msg {
    let status = response.status();
    if !status.is_success() {
        let html = format!(
            "<div>Received {} from server: {}</div>",
            status,
            response.into_body(),
        );

        return Msg::PushLogs(vec![Log::error(html)]);
    }

    let body = response.into_body();
    let response: jrpc::Response<ProjectResultSer> =
        expect!(json::from_str(&body), "response-serde");
    let result = match response {
        jrpc::Response::Ok(r) => r,
        jrpc::Response::Err(err) => {
            return Msg::PushLogs(vec![
                Log::error(format!("<div>received jrpc Error: {:?}</div>", err)),
            ]);
        }
    };

    Msg::RecvProject(result.result.project)
}

impl Renderable<Context, Model> for Model {
    fn view(&self) -> HtmlApp {
        let out = match self.view {
            View::Graph => graph::graph_html(self),
            View::Artifact(ref name) => artifact::view_artifact(self, name),
            View::Edit(id) => edit::view_edit(self, id),
            View::NotFound => html![
                <div class=BOLD,>
                    { "Page not found" }
                </div>
            ],
        };

        nav::view_nav(self, out)
    }
}

fn main() {
    yew::initialize();
    let context = Context {};
    let app: App<_, Model> = App::new(context);
    app.mount_to_body();
    yew::run_loop();
}
