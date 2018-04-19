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

#[macro_use]
extern crate artifact_lib;
extern crate chrono;
#[macro_use]
extern crate ergo_config;
#[macro_use]
extern crate ergo_std;
#[macro_use]
extern crate expect_macro;
extern crate path_abs;
#[macro_use]
extern crate stdweb;
#[macro_use]
extern crate yew;
extern crate yew_router;

mod artifact;
mod dev_prelude;
mod example;
mod graph;
mod name;
mod nav;

use dev_prelude::*;

lazy_static! {
    static ref NAME_URL: Regex = Regex::new(
        &format!(r"(?i)(?:artifacts/)?({})", NAME_VALID_STR)
    ).unwrap();
}

pub(crate) fn router(info: yew_router::RouteInfo) -> Msg {
    let hash = if let Some(h) = info.url.fragment() {
        h
    } else {
        return Msg::Ignore;
    };

    println!("routing hash: {}", hash);
    if hash.to_ascii_lowercase() == "graph" {
        Msg::SetView(View::Graph)
    } else if let Some(cap) = NAME_URL.captures(hash) {
        let name = name!(&cap[1]);
        println!("SetView={}", name);
        Msg::SetView(View::Artifact(name))
    } else {
        println!("ignoring route");
        Msg::Ignore
    }
}

impl Component<Context> for Model {
    type Msg = Msg;
    type Properties = ();

    fn create(_: Self::Properties, context: &mut Env<Context, Self>) -> Self {
        let project: ProjectSer = yaml::from_str(example::YAML).unwrap();
        Model {
            shared: Arc::new(project),
            view: View::Artifact(name!("REQ-completed")),
            router: yew_router::RouterTask::new(context, &router),
            nav: Nav::default(),
        }
    }

    fn update(&mut self, msg: Self::Msg, context: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Msg::SetView(view) => self.view = view,
            Msg::Ignore => return false,
            Msg::ToggleSearch => {
                self.nav.search.on = !self.nav.search.on;
                eprintln!("search toggled to: {}", self.nav.search.on);
            }
            Msg::SetSearch(v) => self.nav.search.value = v,
        }
        true
    }
}

impl Renderable<Context, Model> for Model {
    fn view(&self) -> HtmlApp {
        match self.view {
            View::Graph => graph::graph_html(self),
            View::Artifact(ref name) => artifact::view_artifact(self, name),
        }
    }
}

fn main() {
    yew::initialize();
    let context = Context {};
    let app: App<_, Model> = App::new(context);
    app.mount_to_body();
    yew::run_loop();
}
