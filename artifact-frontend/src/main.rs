#![recursion_limit="128"]

extern crate path_abs;
extern crate chrono;
#[macro_use]
extern crate expect_macro;
#[macro_use]
extern crate yew;
extern crate yew_route;
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
use artifact::ArtifactEdit;

struct Context {
    console: ConsoleService,
}

pub enum View {
    List,
    Artifact(Name),
}

struct Model {
    shared: Arc<ProjectSer>,
    view: View,
    router: yew_route::RouterTask<Context, Model>,
}

enum Msg {
    SetView(View),
    Ignore,
}


lazy_static! {
    static ref NAME_URL: Regex = Regex::new(
        &format!(r"(?i)/?(?:artifacts/)({})", NAME_VALID_STR)
    ).unwrap();
}

pub(crate) fn router(info: yew_route::RouteInfo) -> Msg {
    let url = info.url;
    if let Some(cap) = NAME_URL.captures(url.path()) {
        let name = name!(&cap[1]);
        Msg::SetView(View::Artifact(name))
    } else {
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
            router: yew_route::RouterTask::new(context, &router),
        }
    }

    fn update(&mut self, msg: Self::Msg, context: &mut Env<Context, Self>) -> ShouldRender {
        false
    }
}

impl Renderable<Context, Model> for Model {
    fn view(&self) -> Html<Context, Self> {
        match self.view {
            View::List => {
                html![
                  <h1>{ "List View (unimplemented)" }</h1>
                ]
            }
            View::Artifact(ref name) => {
                let artifact = expect!(self.shared.artifacts.get(name), "FIXME");
                html! [
                  <ArtifactEdit:
                    shared=Some(self.shared.clone()),
                    artifact=Some(artifact.clone()),
                  />
                ]
            }
        }
    }
}

fn main() {
    yew::initialize();
    let context = Context {
        console: ConsoleService,
    };
    let app: App<_, Model> = App::new(context);
    app.mount_to_body();
    yew::run_loop();
}
