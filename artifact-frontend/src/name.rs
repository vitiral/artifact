pub(crate) use artifact_ser::markdown::name_color;
use dev_prelude::*;

pub(crate) fn name_html(model: &Model, name: &Name) -> HtmlApp {
    let color = match model.shared.artifacts.get(name) {
        Some(art) => art.completed.name_color(),
        None => GRAY,
    };

    html![<div><a href=format!("#{}", name), color=color, class=BTN,>
        { name }
    </a></div>]
}
