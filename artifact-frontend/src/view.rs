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

use dev_prelude::*;
use graph;

lazy_static! {
    static ref NAME_URL: Regex =
        Regex::new(&format!(r"(?i)(?:artifacts/)?({})", NAME_VALID_STR)).expect("regex");
    static ref EDIT_URL: Regex = Regex::new(r"(?i)edit/(\d+)").expect("regex");
    static ref REPLACE_TEXT_RE: Regex = Regex::new(&format!(
        r#"(?xim)
        (?:^```dot\s*\n(?P<dot>[\s\S]+?\n)```$)  # graphviz dot rendering
        |({})                       # subname creation
        |({})                       # name reference
        "#,
        name::TEXT_SUB_NAME_STR.as_str(),
        name::TEXT_REF_STR.as_str(),
    )).unwrap();
}

/// The function used for routing urls.
#[allow(needless_pass_by_value)]
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
    let markdown = replace_markdown(model, parent, markdown);
    let value = js!{
        var reader = new commonmark.Parser();
        var writer = new commonmark.HtmlRenderer();
        var parsed = reader.parse(@{markdown.as_ref()});
        return writer.render(parsed);
    };

    let mut md_html = expect!(value.into_string(), "markdown not a string");
    md_html.insert_str(0, "<div>");
    md_html.push_str("</div>");
    let node = expect!(Node::from_html(md_html.trim()), "md-html: {}", md_html);
    VNode::VRef(node)
}

fn replace_markdown<'a, 't>(model: &Model, parent: &'a str, markdown: &'t str) -> Cow<'t, str> {
    use name as web_name;
    let replacer = |cap: &::ergo_std::regex::Captures| -> String {
        if let Some(sub) = cap.name(name::SUB_RE_KEY) {
            replace_markdown_sub(model, parent, sub.as_str())
        } else if let Some(name) = cap.name(name::NAME_RE_KEY) {
            let sub = cap.name(name::NAME_SUB_RE_KEY)
                .map(|s| subname!(s.as_str()));
            web_name::name_markdown(model, &name!(name.as_str()), sub.as_ref())
        } else if let Some(dot) = cap.name("dot") {
            replace_markdown_dot(model, parent, dot.as_str())
        } else {
            panic!("Got unknown match in md: {:?}", cap);
        }
    };
    REPLACE_TEXT_RE.replace_all(markdown, replacer)
}

/// Replace the markdown for a subname declaraction.
fn replace_markdown_sub(model: &Model, parent: &str, sub: &str) -> String {
    let impl_ = model.shared.get_impl(parent, Some(sub));
    let (title, color) = match impl_ {
        Ok(i) => (format!("{:?}", i), BLUE),
        Err(_) => ("Not Implemented".to_string(), RED),
    };

    format!(
        "<span title=\"{}\" style=\"font-weight: bold; color: {}\">\
         {}\
         </span>",
        title, color, sub,
    )
}

fn replace_markdown_dot(model: &Model, parent: &str, dot: &str) -> String {
    let replacer = |cap: &::ergo_std::regex::Captures| -> String {
        if let Some(sub) = cap.name(name::SUB_RE_KEY) {
            graph::subname_dot(model, parent, &subname!(sub.as_str()))
        } else if let Some(name) = cap.name(name::NAME_RE_KEY) {
            let sub = cap.name(name::NAME_SUB_RE_KEY)
                .map(|s| subname!(s.as_str()));
            graph::fullname_dot(model, &name!(name.as_str()), sub.as_ref(), true)
        } else if cap.name("dot").is_some() {
            "**RENDER ERROR: cannot put dot within dot**".into()
        } else {
            panic!("Got unknown match in md: {:?}", cap);
        }
    };
    let dot = REPLACE_TEXT_RE.replace_all(dot, replacer);
    let html = graph::dot_html_string(dot.as_ref());
    format!("\n<html>\n{0}\n</html>\n", html)
}
