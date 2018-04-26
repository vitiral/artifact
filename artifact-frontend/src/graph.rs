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

use stdweb::Value;
use stdweb::unstable::TryFrom;
use stdweb::web::Node;
use yew::virtual_dom::VNode;

use dev_prelude::*;
use name;

/// The small graph at the top of every artifact, displaying it's `partof` and `parts`.
pub(crate) fn artifact_part_html(model: &Model, art: &ArtifactSer) -> HtmlApp {
    // Create node formats
    let mut dot = name_dot(model, &art.name, true);
    for part in &art.parts {
        dot.push_str(&name_dot(model, part, false));
    }
    for part in &art.partof {
        dot.push_str(&name_dot(model, part, false));
    }
    push_connections(&mut dot, art);
    dot_html(&wrap_dot(&model.window, &dot, false))
}

pub(crate) fn graph_html(model: &Model) -> HtmlApp {
    let page = html![<div class=(SM_COL, SM_COL_6, MD_COL_4, LG_COL_2, MR1),>
        <input
         id="search-graph",
         value=model.graph.search.clone(),
         oninput=|e: InputData| Msg::SetGraphSearch(e.value),
         class=INPUT,
         ></input>
         { graph_html_results(model) }
    </div>];

    page
}

pub(crate) fn dot_html_string(dot: &str) -> String {
    let html = js!{
        try {
            var svg = Viz(@{dot});
            return svg;
        } catch ( error ) {
            return (
                "INVALID SVG:\n" + error.message
                + "\n```" + @{dot} + "\n```\n"
            );
        }
    };
    expect!(html.into_string())
}

/// Convert DOT to HTML
pub(crate) fn dot_html(dot: &str) -> HtmlApp {
    let result = js!{
        try {
            var svg = Viz(@{dot});
            var div = document.createElement("div");
            div.innerHTML = svg;
            return { value: div, success: true };
        } catch ( error ) {
            return { value: error.message, success: false };
        }
    };

    let js_div: Value = if let Value::Bool(true) = js!{ return @{&result}.success; } {
        let v = js! { return @{result}.value; };
        v
    } else {
        return html![
            <div color=RED, class=(BOLD, MR1),>
                { "INVALID SVG: " }
            </div>
            <div color=RED, class=(BOLD),>
                { expect!(js!{ return @{result}.value; }.into_string()) }
            </div>
            <textarea readonly=true, value=dot, rows=100, cols=80,>
            </textarea>
        ];
    };

    let node = Node::try_from(js_div).expect("SVG is not a node");

    let svg = VNode::VRef(node);
    html![
        <div>
            { svg }
        </div>
    ]
}

/// The "search graph".
fn graph_html_results(model: &Model) -> HtmlApp {
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
        dot.push_str(&name_dot(model, name, true));

        // push the parts+partof, but only if they are not also
        // in focus (if they are in focus they will be pushed
        // separately)
        for part in &art.parts {
            if !focus.contains_key(part) {
                dot.push_str(&name_dot(model, part, false));
            }
        }

        for part in &art.partof {
            if !focus.contains_key(part) {
                dot.push_str(&name_dot(model, part, false));
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
        dot.push_str(&connect_names_dot(from, to));
    }

    dot_html(&wrap_dot(&model.window, &dot, true))
}

fn connect_names_dot(from: &Name, to: &Name) -> String {
    format!("        \"{}\" -> \"{}\"\n", from.key_str(), to.key_str())
}

fn push_connections(out: &mut String, art: &ArtifactSer) {
    for part in &art.parts {
        out.push_str(&connect_names_dot(&art.name, part));
    }
    for part in &art.partof {
        out.push_str(&connect_names_dot(part, &art.name));
    }
}

/// Put a bunch of dot stuff into the standard graph format.
fn wrap_dot(window: &Window, dot: &str, big: bool) -> String {
    let attrs = if big {
        format!(
            // This is scaling for 1920x1080. I'm not sure how graphviz is measuring an "inch"
            // (how is it getting it?)
            "autosize=false; size=\"{width},{height}!\";",
            width = window.inner_width() / 96,
            height = window.inner_height() / 96,
        )
    } else {
        "randir=LR;".to_string()
    };

    format!(
        r##"
        digraph G {{
        graph [
            margin=0.001; label="";
            {attrs}
        ];

        ////////////////////
        // DOT INSERTED HERE

        {dot}

        ///////////////////
        // END INSERTED DOT

        }}
        "##,
        attrs = attrs,
        dot = dot,
    )
}

pub(crate) fn name_dot(model: &Model, name: &Name, is_focus: bool) -> String {
    fullname_dot(model, name, None, is_focus)
}

pub(crate) fn subname_dot(model: &Model, name: &str, sub: &SubName) -> String {
    let name = match Name::from_str(name) {
    Ok(n) => n,
        Err(_) => return subname_raw(sub, None),
    };
    if model.shared.get_impl(name.as_str(), Some(sub.as_str())).is_err() {
        return subname_raw(sub, None);
    }

    subname_raw(sub, Some(&format!("style=filled; fillcolor=\"{}\"", BLUE)))
}

fn subname_raw(sub: &SubName, attrs: Option<&str>) -> String {
    let attrs = attrs.unwrap_or("penwidth=1.5");
    // FIXME: shape=square?
    format!(
        r##"
        {{
            "{sub_key}" [
                label="{sub}";
                fontsize=12; margin=0.01;
                shape=invhouse;
                {attrs};
            ]
        }}
        "##,
        sub_key = sub.key_str(),
        sub = sub.as_str(),
        attrs = attrs,
    )
}

pub(crate) fn fullname_dot(
    model: &Model,
    name: &Name,
    sub: Option<&SubName>,
    is_focus: bool,
) -> String {
    match model.shared.artifacts.get(name) {
        Some(art) => {
            if let Some(s) = sub {
                if !art.subnames.contains(s) {
                    return dne_name_dot(name, sub);
                }
            }
        }
        None => return dne_name_dot(name, sub),
    };
    let attrs = if is_focus {
        "penwidth=1.5".to_string()
    } else {
        format!("style=filled; fillcolor=\"{}\";", GRAY)
    };

    let (sub, sub_key) = match sub {
        Some(s) => (s.as_str(), s.key_str()),
        None => ("", ""),
    };

    let size = if is_focus { 12 } else { 8 };
    format!(
        r##"
        {{
            "{key}{sub_key}" [
                label="{name}{sub}";
                href="#{name_url}";
                fontcolor="{color}";
                fontsize={size}; margin=0.01;
                shape=invhouse;
                {attrs}
            ]
        }}
        "##,
        key = name.key_str(),
        sub_key = sub_key,
        name = name.as_str(),
        sub = sub,
        name_url = name.key_str().to_ascii_lowercase(),
        color = name::name_color(model, name),
        size = size,
        attrs = attrs,
    )
}

fn dne_name_dot(name: &Name, sub: Option<&SubName>) -> String {
    let (sub, sub_key) = match sub {
        Some(s) => (s.as_str(), s.key_str()),
        None => ("", ""),
    };

    format!(
        r##"
        {{
            "{key}{sub_key}" [
                label=<<b>{name}{sub}</b>>;
                fontcolor=black; style=filled; fillcolor=pink;
                fontsize=12; margin=0.01; shape=invhouse;
                tooltip="Name not found";
            ]
        }}
        "##,
        key = name.key_str(),
        sub_key = sub_key,
        name = name,
        sub = sub,
    )
}
