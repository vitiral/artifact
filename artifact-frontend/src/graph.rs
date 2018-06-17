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

use stdweb::unstable::TryFrom;
use stdweb::web::Node;
use stdweb::Value;
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
    dot_html(&wrap_dot(&model.window, &dot, true))
}

pub(crate) fn graph_html(model: &Model) -> ViewResult {
    let page = html![
        <div class=(SM_COL, SM_COL_6, MD_COL_4, LG_COL_2, MR1),>
             { graph_html_results(model) }
        </div>
    ];

    let nav_extra = Some(html![
        <span>
            <span class=(MR1, FIELD),>{ "| Filter:" }</span>
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

    dot_html(&wrap_dot(&model.window, &dot, false))
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
fn wrap_dot(window: &Window, dot: &str, lr: bool) -> String {
    let attrs = if lr {
        // FIXME: randir isn't working anymore
        "randir=LR;"
    } else {
        ""
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

    let color = if model
        .shared
        .get_impl(name.as_str(), Some(sub.as_str()))
        .is_ok()
    {
        BLUE
    } else {
        RED
    };

    subname_raw(sub, Some(&format!("penwidth=1.5; fontcolor=\"{}\"", color)))
}

fn subname_raw(sub: &SubName, attrs: Option<&str>) -> String {
    let attrs = attrs.unwrap_or("style=filled; fillcolor=\"#DCDEE2\"");
    format!(
        r##"
        {{
            "{sub_key}" [
                label="{sub}";
                fontsize=12; margin=0.15;
                shape=cds;
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
