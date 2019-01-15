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

use stdweb::web::Node;
use yew::virtual_dom::VNode;
use artifact_ser::md_graph;

use dev_prelude::*;
use view::ser_markdown;

/// The small graph at the top of every artifact, displaying it's `partof` and `parts`.
pub(crate) fn artifact_part_html(model: &Model, art: &ArtifactSer) -> HtmlApp {
    let md = ser_markdown(model);
    let dot = md_graph::artifact_part_dot(&md, art);
    dot_html(&dot)
}

pub(crate) fn graph_html(model: &Model) -> ViewResult {
    let page = html![
        <div class=(SM_COL, SM_COL_6, MD_COL_4, LG_COL_2, MR1),>
             { graph_html_results(model) }
        </div>
    ];

    let nav_extra = Some(html![
        <span>
            <span class=(MR1, FIELD),>{ "Filter:" }</span>
            <input id="graph-nav-filter",
             type="text",
             class=(FIELD),
             value=model.graph.search.clone(),
             oninput=|e| Msg::SetGraphSearch(e.value),
            >
            </input>
        </span>
    ]);

    ViewResult {
        page,
        nav_extra,
    }
}

/// Get the dot html for untrusted dot.
pub(crate) fn dot_html_string(dot: &str) -> String {
    let html = js!{
        try {
            var svg = Viz(@{dot});
            return svg;
        } catch ( error ) {
            return (
                "<h2>ERROR: Invalid SVG</h2>\n" + error.message
                + "\n<pre><code class=\"language-//\">" + @{dot} + "\n</code></pre>\n"
            );
        }
    };
    expect!(html.into_string())
}

/// Convert DOT to HTML
pub(crate) fn dot_html(dot: &str) -> HtmlApp {
    let dot_string = dot_html_string(dot);
    let dot_string = format!("<div>{}</div>", dot_string);
    let node = expect!(Node::from_html(&dot_string), "invalid html");
    let svg = VNode::VRef(node);
    html![
        <div>
       { svg }
        </div>
    ]
}

/// The "search graph".
fn graph_html_results(model: &Model) -> HtmlApp {
    let md = ser_markdown(model);

    let re = match parse_regex(&model.graph.search) {
        Ok(r) => r,
        Err(e) => return e,
    };
    let mut dot = String::new();

    let focus: HashMap<&Name, &ArtifactSer> = model
        .shared
        .artifacts
        .iter()
        .filter(|(n, _)| re.is_match(n.as_str()))
        .collect();

    for (name, art) in &focus {
        dot.push_str(&md_graph::name_dot(&md, name, true));

        // push the parts+partof, but only if they are not also
        // in focus (if they are in focus they will be pushed
        // separately)
        for part in &art.parts {
            if !focus.contains_key(part) {
                dot.push_str(&md_graph::name_dot(&md, part, false));
            }
        }

        for part in &art.partof {
            if !focus.contains_key(part) {
                dot.push_str(&md_graph::name_dot(&md, part, false));
            }
        }
    }

    let mut connections: HashSet<(&Name, &Name)> = HashSet::new();

    for (name, art) in &focus {
        for part in &art.parts {
            connections.insert((name, part));
        }
        for part in &art.partof {
            connections.insert((part, name));
        }
    }

    for (from, to) in connections {
        dot.push_str(&md_graph::connect_names_dot(from, to));
    }

    dot_html(&md_graph::wrap_dot(&dot, false))
}
