extern crate path_abs;
extern crate chrono;
#[macro_use]
extern crate yew;
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

struct Model {
    value: i64,
    /// Shared read-only data with sub-components.
    shared: Arc<ProjectSer>,
    viewing: Name,
    example_json: String,
}

enum Msg {
    Increment,
    Decrement,
    Bulk(Vec<Msg>),
}

impl Component<Context> for Model {
    type Msg = Msg;
    type Properties = ();

    fn create(_: Self::Properties, _: &mut Env<Context, Self>) -> Self {
        let project: ProjectSer = yaml::from_str(example::YAML).unwrap();
        Model {
            value: 0,
            shared: Arc::new(project),
            viewing: name!("REQ-completed"),
            example_json: js!{return "REPLACE FOR EXAMPLE"}
                .into_string().unwrap_or("couldn't unwrap".into()),
        }
    }

    fn update(&mut self, msg: Self::Msg, context: &mut Env<Context, Self>) -> ShouldRender {
        match msg {
            Msg::Increment => {
                self.value = self.value + 1;
                context.console.log("plus one");
            }
            Msg::Decrement => {
                self.value = self.value - 1;
                context.console.log("minus one");
            }
            Msg::Bulk(list) => {
                for msg in list {
                    self.update(msg, context);
                }
            }
        }
        true
    }
}

impl Renderable<Context, Model> for Model {
    fn view(&self) -> Html<Context, Self> {
        let artifact = self.shared.artifacts.get(&self.viewing).unwrap();
        html! [
            <div><p>{format!("EXAMPLE JSON: {}", self.example_json)}</p></div>
            <div>
                <nav class="menu",>
                    <button onclick=|_| Msg::Increment,>{ "Increment" }</button>
                    <button onclick=|_| Msg::Decrement,>{ "Decrement" }</button>
                    <button onclick=|_| Msg::Bulk(vec!(Msg::Increment, Msg::Increment)),>{ "Increment Twice" }</button>
                </nav>
                <p>{ self.value }</p>
                <p>{ Local::now() }</p>
            </div>
            <div>
                <h1>{ format!("Artifact: {}", artifact.name) }</h1>
                <p><b>{"Parts:"}</b></p>
                { view_parts(artifact) }
                <ArtifactEdit: shared=Some(self.shared.clone()), artifact=Some(artifact.clone()), />
            </div>
        ]
    }
}

fn view_parts(artifact: &ArtifactSer) -> Html<Context, Model> {
    let view_part = |name: &Name| html![
        <li>{name}</li>
    ];
    html![
        <ul>{ for artifact.parts.iter().map(view_part) }</ul>
    ]
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
