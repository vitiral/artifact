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

use dev_prelude::*;
use name;

use stdweb::Value;
use stdweb::web::Node;
use stdweb::unstable::TryFrom;
use yew::virtual_dom::VNode;

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
    dot_html(&wrap_dot(&dot))
}

pub(crate) fn graph_html(model: &Model) -> HtmlApp {
    let mut dot = String::new();

    for name in model.shared.artifacts.keys() {
        dot.push_str(&name_dot(model, name, false));
    }

    for art in model.shared.artifacts.values() {
        push_connections(&mut dot, art);
    }

    dot_html(&wrap_dot(&dot))
}

/// Convert DOT to HTML
fn dot_html(dot: &str) -> HtmlApp {
    let result = js!{
        try {
            var svg = Viz(@{dot});
            console.log("SVG\n" + svg);
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
            <textarea readonly=true, value=dot.to_owned(), rows=100, cols=80,>
            </textarea>
        ];
    };

    let node = Node::try_from(js_div).expect("SVG is not a node");

    let svg = VNode::VRef(node);
    html![
        <h1>{ "Graph View" }</h1>
        { svg }
    ]
}

fn push_connections(out: &mut String, art: &ArtifactSer) {
    fn connect_names_dot(from: &Name, to: &Name) -> String {
        format!("        \"{}\" -> \"{}\"\n", from.key_str(), to.key_str())
    }

    for part in &art.parts {
        out.push_str(&connect_names_dot(&art.name, part));
    }
    for part in &art.partof {
        out.push_str(&connect_names_dot(part, &art.name));
    }
}

/// Put a bunch of dot stuff into the standard graph format.
fn wrap_dot(dot: &str) -> String {
    format!(
        r##"
        digraph G {{ graph [rankdir=LR, fontsize=14, margin=0.001];
        subgraph cluster_family {{
        margin=4; label=<<b>related artifacts</b>>

        ////////////////////
        // DOT INSERTED HERE

        {dot}

        ///////////////////
        // END INSERTED DOT

        }}
        }}
        "##,
        dot=dot
    )
}

fn name_dot(model: &Model, name: &Name, is_focus: bool) -> String {
    if !model.shared.artifacts.contains_key(name) {
        return dne_name_dot(name);
    }
    let attrs = if is_focus {
        "penwidth=1.5; style=filled; fillcolor=cyan;"
    } else {
        ""
    };
    format!(
        r##"
        {{
            "{key}" [
                label=<<b>{name}</b>>;
                href="#{name_url}";
                fontcolor="{color}";
                fontsize=12; margin=0.01; shape=invhouse;
                {attrs}
            ]
        }}
        "##,
        key=name.key_str(),
        name=name.as_str(),
        name_url=name.as_str().to_ascii_lowercase(),
        color=name::name_color(model, name),
        attrs=attrs,
    )
}

fn dne_name_dot(name: &Name) -> String {
    format!(
        r##"
        {{
            "{key}" [
                label=<<b>{name}</b>>;
                fontcolor=black; style=filled; fillcolor=pink;
                fontsize=12; margin=0.01; shape=invhouse;
                tooltip="Name not found";
            ]
        }}
        "##,
        key=name.key_str(),
        name=name,
    )
}

