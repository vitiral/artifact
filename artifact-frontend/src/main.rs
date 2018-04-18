#![recursion_limit="128"]

extern crate path_abs;
extern crate chrono;
#[macro_use]
extern crate expect_macro;
#[macro_use]
extern crate yew;
extern crate yew_router;
#[macro_use]
extern crate artifact_lib;
#[macro_use]
extern crate ergo_std;
#[macro_use]
extern crate ergo_config;
#[macro_use]
extern crate stdweb;

mod artifact;
mod example;
mod dev_prelude;

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
    if let Some(cap) = NAME_URL.captures(hash) {
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
        }
    }

    fn update(&mut self, msg: Self::Msg, context: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Msg::SetView(view) => {
                println!("Setting view: {:?}", view);
                self.view = view;
                true
            }
            Msg::Ignore => {
                false
            }
        }
    }
}

impl Renderable<Context, Model> for Model {
    fn view(&self) -> HtmlApp {
        match self.view {
            View::List => {
                html![
                  <h1>{ "List View" }</h1>
                ]
            }
            View::Artifact(ref name) => {
                artifact::view_artifact(self, name)
            }
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
