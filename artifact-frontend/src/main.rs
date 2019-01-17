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
#![recursion_limit = "256"]
#![allow(unknown_lints)]

#[macro_use]
extern crate artifact_ser;
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

use yew::prelude::*;

mod artifact;
mod dev_prelude;
mod edit;
mod example;
mod fetch;
mod graph;
mod name;
mod nav;
mod view;

use dev_prelude::*;

impl Component<Context> for Model {
    type Message = Msg;
    type Properties = ();

    fn create(_: Self::Properties, context: &mut Env<Context, Self>) -> Self {
        let project: ProjectSer = expect!(yaml::from_str(example::YAML));
        let router = yew_simple::RouterTask::new(context, &view::router_fn);
        let url = router.current_url();

        let mut model = Model {
            web_type: WebType::Static,
            shared: Arc::new(project),
            view: View::from_hash(&url.fragment().unwrap_or_default()),
            router: Arc::new(router),
            nav: Nav::default(),
            graph: Graph::default(),
            fetch_task: None,
            console: Arc::new(ConsoleService::new()),
            logs: Logs::default(),
            window: ::stdweb::web::window(),
            editing: IndexMap::new(),
            updating: IndexMap::new(),
        };
        model.nav.search.on = true;
        model.nav.editing.on = true;
        fetch::start_fetch_initial(&mut model, context);
        model
    }

    fn update(&mut self, msg: Self::Message, context: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Msg::Batch(mut batch) => {
                let count: usize = batch
                    .drain(..)
                    .map(|msg| update_model(self, msg, context) as usize)
                    .sum();
                count != 0
            }
            msg => update_model(self, msg, context),
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

        Msg::RecvInitial(init) => {
            if let Some(project) = init.project {
                model.shared = Arc::new(project);
            }
            model.web_type = init.web_type;
            model.fetch_task = None;

            if model.web_type != WebType::Static {
                fetch::start_fetch_project(model, context, false);
            }
        }
        Msg::FetchProject { reload } => return fetch::start_fetch_project(model, context, reload),
        Msg::SendUpdate(ids) => return fetch::start_send_update(model, context, ids),
        Msg::RecvProject(jid, project) => fetch::handle_recv_project(model, &jid, project),
        Msg::RecvError(logs) => {
            model.push_logs(logs);
            model.fetch_task = None;
        }

        Msg::PushLogs(logs) => model.push_logs(logs),
        Msg::ClearLogs(clear) => clear_logs(model, clear),

        Msg::EditArtifact(id, field) => edit::handle_edit_artifact(model, id, field),
        Msg::StartEdit(id, ty) => edit::handle_start_edit(model, id, &ty),
        Msg::StopEdit(id) => model.complete_editing(id),
        Msg::Batch(_) => panic!("batch within a batch"),
    }
    true
}

fn clear_logs(model: &mut Model, clear: ClearLogs) {
    match clear {
        ClearLogs::Error(mut ids) => {
            for id in ids.drain(..) {
                model.logs.error.remove(&id);
            }
        }
        ClearLogs::ErrorAll => {
            model.logs.error.clear();
        }
    }
}

impl Renderable<Context, Model> for Model {
    fn view(&self) -> HtmlApp {
        let view = match self.view {
            View::Graph => graph::graph_html(self),
            View::Artifact(ref name) => artifact::view_artifact(self, name),
            View::Edit(id) => edit::view_edit(self, id),
            View::NotFound => {
                let page = html![
                    <div class=BOLD,>
                        { "Page not found" }
                    </div>
                ];
                ViewResult {
                    page,
                    nav_extra: None,
                }
            }
        };

        nav::view_nav(self, view)
    }
}

fn main() {
    yew::initialize();
    let context = Context {};
    let app: App<_, Model> = App::new(context);
    app.mount_to_body();
    yew::run_loop();
}
