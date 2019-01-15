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

use dev_prelude::*;
use name::*;
use ser::*;
use markdown::{GRAY, BLUE, RED, SerMarkdown, name_color};

pub fn artifact_part_dot(md: &SerMarkdown, art: &ArtifactSer) -> String {
    let mut dot = name_dot(md, &art.name, true);
    for part in &art.parts {
        dot.push_str(&name_dot(md, part, false));
    }
    for part in &art.partof {
        dot.push_str(&name_dot(md, part, false));
    }
    push_connections(&mut dot, art);
    wrap_dot(&dot, true)
}

pub fn connect_names_dot(from: &Name, to: &Name) -> String {
    format!("        \"{}\" -> \"{}\"\n", from.key_str(), to.key_str())
}

pub fn push_connections(out: &mut String, art: &ArtifactSer) {
    for part in &art.parts {
        out.push_str(&connect_names_dot(&art.name, part));
    }
    for part in &art.partof {
        out.push_str(&connect_names_dot(part, &art.name));
    }
}

/// Put a bunch of dot stuff into the standard graph format.
pub fn wrap_dot(dot: &str, lr: bool) -> String {
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

pub fn name_dot(md: &SerMarkdown, name: &Name, is_focus: bool) -> String {
    fullname_dot(md, name, None, is_focus)
}

pub fn subname_dot(md: &SerMarkdown, name: &str, sub: &SubName) -> String {
    let name = match Name::from_str(name) {
        Ok(n) => n,
        Err(_) => return subname_raw(sub, None),
    };

    let color = if md.project
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

pub fn fullname_dot(
    md: &SerMarkdown,
    name: &Name,
    sub: Option<&SubName>,
    is_focus: bool,
) -> String {
    match md.project.artifacts.get(name) {
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
        color = name_color(&md.project, name),
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

