use dev_prelude::*;
use ser::*;
use std::io;

use name::*;
use super::Completed;
use md_graph;

pub const GRAY: &str = "#DCDEE2";
pub const OLIVE: &str = "#3DA03D";
pub const BLUE: &str = "#0074D9";
pub const ORANGE: &str = "#FF851B";
pub const RED: &str = "#FF4136";
pub const PURPLE: &str = "#B10DC9";


impl ProjectSer {
    pub fn to_markdown(&self, w: &mut io::Write) -> io::Result<()> {
        for artifact in self.artifacts.values() {
            artifact.to_markdown(w, self)?;
        }
        Ok(())
    }
}

impl ArtifactSer {
    pub fn to_markdown(&self, w: &mut io::Write, project: &ProjectSer) -> io::Result<()> {
        write!(w, "# {0} {{#{0}}}\n", self.name)?;
        write!(w, "```dot\n{}\n```\n", md_graph::artifact_part_dot(project, self))?;
        write!(w, "\n- file: {}\n", self.file)?;
        write!(w, "\n- impl: {}\n", self.impl_)?;
        write!(w, "\n- {}\n", self.completed)?;
        write!(w, "\n")?;
        write!(w, "{}\n\n", replace_markdown(project, self.name.as_str(), &self.text))?;
        Ok(())
    }
}


lazy_static! {
    static ref NAME_URL: Regex =
        Regex::new(&format!(r"(?i)(?:artifacts/)?({})", NAME_VALID_STR)).expect("regex");
    static ref EDIT_URL: Regex = Regex::new(r"(?i)edit/(\d+)").expect("regex");
    static ref REPLACE_TEXT_RE: Regex = Regex::new(&format!(
        r#"(?xim)
        |({})                       # subname creation
        |({})                       # name reference
        "#,
        TEXT_SUB_NAME_STR.as_str(),
        TEXT_REF_STR.as_str(),
    )).unwrap();
}

/// Handle specialized markdown syntax, replacing with standard markdown.
///
/// `parent` is the parent's name, which may or may not exist/be-valid.
pub fn replace_markdown<'a, 't>(project: &ProjectSer, parent: &'a str, markdown: &'t str) -> Cow<'t, str> {
    use name as web_name;
    let replacer = |cap: &::ergo_std::regex::Captures| -> String {
        if let Some(sub) = cap.name(SUB_RE_KEY) {
            replace_markdown_sub(project, parent, sub.as_str())
        } else if let Some(name) = cap.name(NAME_RE_KEY) {
            let sub = cap.name(NAME_SUB_RE_KEY)
                .map(|s| subname!(s.as_str()));
            name_markdown(project, &name!(name.as_str()), sub.as_ref())
        } else if let Some(dot) = cap.name("dot") {
            replace_markdown_dot(project, parent, dot.as_str())
        } else {
            panic!("Got unknown match in md: {:?}", cap);
        }
    };
    REPLACE_TEXT_RE.replace_all(markdown, replacer)
}


// impl View {
//     pub(crate) fn from_hash(hash: &str) -> View {
//         if hash.to_ascii_lowercase() == "graph" || hash == "" {
//             View::Graph
//         } else if let Some(cap) = NAME_URL.captures(hash) {
//             let name = name!(&cap[1]);
//             View::Artifact(name)
//         } else if let Some(cap) = EDIT_URL.captures(hash) {
//             let id = match usize::from_str(&cap[1]) {
//                 Ok(id) => id,
//                 Err(_) => return View::NotFound,
//             };
//             View::Edit(id)
//         } else {
//             View::NotFound
//         }
//     }
// }

/// Replace the markdown for a subname declaraction.
fn replace_markdown_sub(project: &ProjectSer, parent: &str, sub: &str) -> String {
    let impl_ = project.get_impl(parent, Some(sub));
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

fn replace_markdown_dot<'a, 't>(project: &ProjectSer, parent: &'a str, dot: &'t str)
        -> String {
    let replacer = |cap: &::ergo_std::regex::Captures| -> String {
        if let Some(sub) = cap.name(SUB_RE_KEY) {
            md_graph::subname_dot(project, parent, &subname!(sub.as_str()))
        } else if let Some(name) = cap.name(NAME_RE_KEY) {
            let sub = cap.name(NAME_SUB_RE_KEY)
                .map(|s| subname!(s.as_str()));
            md_graph::fullname_dot(project, &name!(name.as_str()), sub.as_ref(), true)
        } else if cap.name("dot").is_some() {
            "**RENDER ERROR: cannot put dot within dot**".into()
        } else {
            panic!("Got unknown match in md: {:?}", cap);
        }
    };
    return REPLACE_TEXT_RE.replace_all(dot, replacer).to_string();
}

/// Used for inserting into markdown, etc.
pub(crate) fn name_markdown(project: &ProjectSer, name: &Name, sub: Option<&SubName>) -> String {
    if let Some(art) = project.artifacts.get(name) {
        if let Some(s) = sub {
            return name_subname_markdown(project, art, name, s);
        } else {
            name_html_raw(
                name.as_str(),
                None,
                name_color(project, name),
                CssFont::Bold,
                name.key_str(),
                Some(name.key_str()),
            )
        }
    } else {
        name_html_raw(
            name.as_str(),
            None,
            PURPLE,
            CssFont::Italic,
            &format!("{} not found", name.key_str()),
            None,
        )
    }
}

fn name_subname_markdown(project: &ProjectSer, art: &ArtifactSer, name: &Name, sub: &SubName) -> String {
    if art.subnames.contains(sub) {
        name_html_raw(
            name.as_str(),
            Some(sub.as_str()),
            name_color(project, name),
            CssFont::Bold,
            &format!("{}{}", name.key_str(), sub.key_str()),
            Some(name.key_str()),
        )
    } else {
        name_html_raw(
            name.as_str(),
            Some(sub.as_str()),
            PURPLE,
            CssFont::Italic,
            &format!("{}{} not found", name.key_str(), sub.key_str()),
            None,
        )
    }
}

pub fn name_color(project: &ProjectSer, name: &Name) -> &'static str {
    match project.artifacts.get(name) {
        Some(art) => completed_color(&art.completed),
        None => GRAY,
    }
}

fn name_html_raw(
    name: &str,
    sub: Option<&str>,
    color: &str,
    font: CssFont,
    title: &str,
    href: Option<&str>,
) -> String {
    let href = match href {
        Some(h) => format!("href=\"#{}\"", h),
        None => "".into(),
    };
    let sub = match sub {
        Some(s) => s,
        None => "",
    };

    format!(
        r##"<a style="{}color: {}" title="{}" {}>{}{}</a>"##,
        font.as_css(),
        color,
        title,
        href,
        name,
        sub
    )
}

#[derive(Debug, Copy, Clone)]
pub enum CssFont {
    Plain,
    Bold,
    Italic,
}

impl CssFont {
    pub fn as_css(&self) -> &'static str {
        match *self {
            CssFont::Plain => "",
            CssFont::Bold => "font-weight: bold; ",
            CssFont::Italic => "font-style: italic; ",
        }
    }
}

fn completed_color(c: &Completed) -> &'static str {
    match c.spc_points() + c.tst_points() {
        0 => RED,
        1 | 2 => ORANGE,
        3 | 4 => BLUE,
        5 => OLIVE,
        _ => panic!("invalid name points"),
    }
}
