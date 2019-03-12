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
use yew_simple;

use crate::dev_prelude::*;
use crate::graph;
use artifact_ser;

lazy_static! {
    static ref NAME_URL: Regex = expect!(Regex::new(&format!(
        r"(?i)(?:artifacts/)?({})",
        NAME_VALID_STR
    )));
    static ref EDIT_URL: Regex = expect!(Regex::new(r"(?i)edit/(\d+)"));
    static ref REPLACE_TEXT_RE: Regex = expect!(Regex::new(
        r#"(?xim)
        (?:^```dot\s*\n(?P<dot>[\s\S]+?\n)```$)  # graphviz dot rendering
        "#,
    ));
}

/// The function used for routing urls.
#[allow(clippy::needless_pass_by_value)]
pub(crate) fn router_fn(info: yew_simple::RouteInfo) -> Msg {
    let hash = info.url.fragment().unwrap_or_default();
    let view = View::from_hash(hash);
    Msg::SetView(view)
}

impl View {
    pub(crate) fn from_hash(hash: &str) -> View {
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
}

/// Render the markdown correctly.
///
/// `parent` is the parent's name, which may or may not exist/be-valid.
pub(crate) fn markdown_html(model: &Model, parent: &str, markdown: &str) -> HtmlApp {
    let md = ser_markdown(model);
    let markdown = md.replace_markdown(parent, markdown);
    let markdown = replace_markdown(&markdown).to_string();
    let value = js! {
        var reader = new commonmark.Parser();
        var writer = new commonmark.HtmlRenderer();
        var parsed = reader.parse(@{markdown});
        return writer.render(parsed);
    };

    let mut md_html = expect!(value.into_string(), "markdown not a string");
    md_html.insert_str(0, "<div>");
    md_html.push_str("</div>");
    let node = expect!(Node::from_html(md_html.trim()), "md-html: {}", md_html);
    VNode::VRef(node)
}

/// Return the default SerMarkdown object for the frontend.
pub(crate) fn ser_markdown(model: &Model) -> artifact_ser::markdown::SerMarkdown<'_> {
    use artifact_ser::markdown::*;
    let settings = SerMarkdownSettings {
        code_url: model.shared.settings.code_url.clone(),
        family: SettingsMdFamily::Dot,
        dot: SettingsMdDot::Ignore,
        name_prefix: "".to_string(),
        // md_plain: false,
        // md_details: SettingsMdDetails::default(),
    };

    SerMarkdown::with_settings(&model.shared, settings)
}

fn replace_markdown<'t>(markdown: &'t str) -> Cow<'t, str> {
    let replacer = |cap: &::ergo_std::regex::Captures<'_>| -> String {
        if let Some(dot) = cap.name("dot") {
            replace_markdown_dot(dot.as_str())
        } else {
            panic!("Got unknown match in md: {:?}", cap);
        }
    };
    REPLACE_TEXT_RE.replace_all(markdown, replacer)
}

fn replace_markdown_dot(dot: &str) -> String {
    let html = graph::dot_html_string(dot);
    format!("\n<html>\n{0}\n</html>\n", html)
}
