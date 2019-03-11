use crate::dev_prelude::*;
use crate::ser::*;
use std::io;

use super::{Completed, SettingsMdDot, SettingsMdFamily};
use crate::md_graph;
use crate::name::*;

pub const GRAY: &str = "#DCDEE2";
pub const OLIVE: &str = "#3DA03D";
pub const BLUE: &str = "#0074D9";
pub const ORANGE: &str = "#FF851B";
pub const RED: &str = "#FF4136";
pub const PURPLE: &str = "#B10DC9";

pub const DOT_RE_KEY: &str = "dot";
pub const DOT_PRE_RE_KEY: &str = "dot_pre";
pub const DOT_POST_RE_KEY: &str = "dot_post";

lazy_static! {
    // TODO: make this instead to allow for extra tags
    // (?:^```dot [\s\S]*?\n(?P<{}>[\s\S]+?\n)```$)
    pub static ref TEXT_DOT_STR: String = format!(r#"
        (?:^(?P<{}>```dot\s*)\n(?P<{}>[\s\S]+?\n)(?P<{}>```)$)
        "#,
        DOT_PRE_RE_KEY,
        DOT_RE_KEY,
        DOT_POST_RE_KEY,
    );

    static ref NAME_URL: Regex =
        Regex::new(&format!(r"(?i)(?:artifacts/)?({})", NAME_VALID_STR)).expect("regex");
    static ref EDIT_URL: Regex = Regex::new(r"(?i)edit/(\d+)").expect("regex");
    static ref REPLACE_TEXT_RE: Regex = Regex::new(&format!(
        r#"(?xim)
        |({})                       # subname creation
        |({})                       # name reference
        |({})                     # dot replacement
        "#,
        TEXT_SUB_NAME_STR.as_str(),
        TEXT_REF_STR.as_str(),
        TEXT_DOT_STR.as_str()
    ))
    .unwrap();
}

#[derive(Debug, PartialEq)]
pub struct SerMarkdown<'a> {
    pub(crate) project: &'a ProjectSer,
    settings: SerMarkdownSettings,
}

#[derive(Debug, Default, Clone, Eq, PartialEq, Serialize, Deserialize)]
pub struct SerMarkdownSettings {
    pub code_url: Option<String>,
    pub family: SettingsMdFamily,
    pub dot: SettingsMdDot,
    pub name_prefix: String,
    // pub md_plain: bool,
    // pub md_details: SettingsMdDetails,
}

impl<'a> SerMarkdown<'a> {
    pub fn new(project: &'a ProjectSer) -> SerMarkdown<'_> {
        Self::with_settings(project, SerMarkdownSettings::default())
    }

    pub fn with_settings(
        project: &'a ProjectSer,
        settings: SerMarkdownSettings,
    ) -> SerMarkdown<'_> {
        SerMarkdown {
            project: project,
            settings: settings,
        }
    }

    /// Export the whole project as markdown.
    pub fn to_markdown(&'a self, w: &mut dyn io::Write) -> io::Result<()> {
        let export = &self.project.settings.export;

        if let Some(ref header) = export.md_header {
            write!(w, "{}\n", header)?;
        }

        if export.md_toc {
            self.to_markdown_toc(w)?;
        }

        for artifact in self.project.artifacts.values() {
            self.art_to_markdown(w, artifact)?;
        }
        Ok(())
    }

    fn to_markdown_toc(&self, w: &mut dyn io::Write) -> io::Result<()> {
        write!(w, "{}# Table Of Contents\n", self.settings.name_prefix)?;
        for name in self.project.artifacts.keys() {
            write!(w, "- {}\n", self.name_markdown(name, None))?;
        }
        write!(w, "\n\n")?;
        Ok(())
    }

    pub fn art_to_markdown(&self, w: &mut dyn io::Write, artifact: &ArtifactSer) -> io::Result<()> {
        macro_rules! write_html_line {
            ($section:expr, $content:expr) => {{
                write!(w, "<b>{}:</b> {}<br>\n", $section, $content)?;
            }};
        }

        write!(w, "{}# {}\n", self.settings.name_prefix, artifact.name)?;
        self.tag_details_begin(w, "metadata")?;
        self.art_to_markdown_family(w, artifact)?;
        write_html_line!("file", self.html_file_url(&artifact.file));

        let impl_ = match artifact.impl_ {
            ImplSer::Done(ref d) => d.clone(),
            ImplSer::Code(ImplCodeSer {
                primary: Some(ref c),
                ..
            }) => self.html_code_url(c),
            _ => "<i>not implemented</i>".to_string(),
        };
        write_html_line!("impl", impl_);

        write!(
            w,
            "<b>spc:</b>{:.2}&nbsp;&nbsp;<b>tst:</b>{:.2}<br>\n",
            artifact.completed.spc * 100.0,
            artifact.completed.tst * 100.0
        )?;
        write!(w, "<hr>\n")?;
        self.tag_details_end(w)?;
        write!(
            w,
            "{}\n\n",
            self.replace_markdown(artifact.name.as_str(), &artifact.text)
        )?;
        Ok(())
    }

    fn art_to_markdown_family(
        &self,
        w: &mut dyn io::Write,
        artifact: &ArtifactSer,
    ) -> io::Result<()> {
        match self.settings.family {
            SettingsMdFamily::List => {
                macro_rules! write_section {
                    ($section:ident) => {{
                        if artifact.$section.is_empty() {
                            write!(w, "<b>{}:</b> <i>none</i></a><br>\n", stringify!($section))?;
                        } else {
                            write!(w, "<b>{}:</b><br>\n", stringify!($section))?;
                            for name in &artifact.$section {
                                write!(w, "<li>{}</li>\n", self.name_markdown(&name, None))?;
                            }
                        }
                    }};
                }
                write_section!(partof);
                write_section!(parts);
            }
            SettingsMdFamily::Dot => {
                self.tag_details_begin(w, "dot-graph")?;
                let (dot_pre, dot_post) = match self.settings.dot {
                    SettingsMdDot::ReplaceBraces { ref pre, ref post } => {
                        (pre.as_str(), post.as_str())
                    }
                    _ => ("```dot", "```"),
                };
                write!(
                    w,
                    "{pre}\n{dot}\n{post}\n",
                    pre = dot_pre,
                    dot = md_graph::artifact_part_dot(self, artifact),
                    post = dot_post,
                )?;
                self.tag_details_end(w)?;
            }
        }
        Ok(())
    }

    /// Handle specialized markdown syntax, replacing with standard markdown.
    ///
    /// `parent` is the parent's name, which may or may not exist/be-valid.
    pub fn replace_markdown<'p, 'm>(&'a self, parent: &'p str, markdown: &'m str) -> Cow<'m, str> {
        let replacer = |cap: &::ergo_std::regex::Captures<'_>| -> String {
            if let Some(sub) = cap.name(SUB_RE_KEY) {
                self.replace_markdown_sub(parent, sub.as_str())
            } else if let Some(name) = cap.name(NAME_RE_KEY) {
                let sub = cap.name(NAME_SUB_RE_KEY).map(|s| subname!(s.as_str()));
                self.name_markdown(&name!(name.as_str()), sub.as_ref())
            } else if cap.name(DOT_RE_KEY).is_some() {
                self.replace_markdown_dot(parent, &cap)
            } else {
                panic!("Got unknown match in md: {:?}", cap);
            }
        };
        REPLACE_TEXT_RE.replace_all(markdown, replacer)
    }

    /// Replace the markdown for a subname declaraction.
    fn replace_markdown_sub(&'a self, parent: &str, sub: &str) -> String {
        let (title, color, href): (String, &'static str, Option<String>) =
            match self.project.get_impl(parent, Some(sub)) {
                Ok(c) => {
                    let href = self.format_code_url_maybe(c);
                    (format!("{:?}", c), BLUE, href)
                }
                Err(_) => ("Not Implemented".to_string(), RED, None),
            };

        match href {
            Some(href) => format!(
                "<a title=\"{}\" style=\"color: {}\" \
                 href=\"{}\">\
                 <b>{}</b>\
                 </a>",
                title, color, href, sub,
            ),
            None => format!(
                "<span title=\"{}\" style=\"color: {}\">\
                 <b><i>{}</i></b>\
                 </span>",
                title, color, sub,
            ),
        }
    }

    fn replace_markdown_dot<'p, 'd>(
        &'a self,
        parent: &'p str,
        cap: &::ergo_std::regex::Captures<'_>,
    ) -> String {
        let replacer = |cap: &::ergo_std::regex::Captures<'_>| -> String {
            if let Some(sub) = cap.name(SUB_RE_KEY) {
                md_graph::subname_dot(self, parent, &subname!(sub.as_str()))
            } else if let Some(name) = cap.name(NAME_RE_KEY) {
                let sub = cap.name(NAME_SUB_RE_KEY).map(|s| subname!(s.as_str()));
                md_graph::fullname_dot(self, &name!(name.as_str()), sub.as_ref(), true)
            } else if cap.name(DOT_RE_KEY).is_some() {
                "**RENDER ERROR: cannot put dot within dot**".into()
            } else {
                panic!("Got unknown match in md: {:?}", cap);
            }
        };
        let dot_pre = expect!(cap.name(DOT_PRE_RE_KEY)).as_str();
        let dot_post = expect!(cap.name(DOT_POST_RE_KEY)).as_str();
        let dot = expect!(cap.name(DOT_RE_KEY)).as_str();
        let dot = REPLACE_TEXT_RE.replace_all(dot, replacer).to_string();

        let (dot_pre, dot_post) = match self.settings.dot {
            SettingsMdDot::Ignore => (dot_pre, dot_post),
            SettingsMdDot::RemoveBraces => ("", ""),
            SettingsMdDot::ReplaceBraces { ref pre, ref post } => (pre.as_str(), post.as_str()),
        };
        format!(
            "{pre}\n{dot}\n{post}",
            pre = dot_pre,
            post = dot_post,
            dot = dot,
        )
    }

    /// Used for inserting into markdown, etc.
    pub(crate) fn name_markdown(&'a self, name: &Name, sub: Option<&SubName>) -> String {
        if let Some(art) = self.project.artifacts.get(name) {
            if let Some(s) = sub {
                return self.name_subname_markdown(art, name, s);
            } else {
                name_html_raw(
                    name.as_str(),
                    None,
                    name_color(&self.project, name),
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

    fn name_subname_markdown(&'a self, art: &ArtifactSer, name: &Name, sub: &SubName) -> String {
        if art.subnames.contains(sub) {
            name_html_raw(
                /*name= */ name.as_str(),
                /*sub=  */ Some(sub.as_str()),
                /*color=*/ name_color(&self.project, name),
                /*font= */ CssFont::Bold,
                /*title=*/ &format!("{}{}", name.key_str(), sub.key_str()),
                /*href= */ Some(name.key_str()),
            )
        } else {
            name_html_raw(
                /*name= */ name.as_str(),
                /*sub=  */ Some(sub.as_str()),
                /*color=*/ PURPLE,
                /*font= */ CssFont::Italic,
                /*title=*/ &format!("{}{} not found", name.key_str(), sub.key_str()),
                /*href= */ None,
            )
        }
    }

    /// Always get correct markdown for a file.
    pub fn html_file_url(&self, file: &str) -> String {
        let file_codeloc = CodeLocSer::with_file(file.to_string());
        let trimmed_file = self.project.settings.trim_base(file);
        match self.format_code_url_maybe(&file_codeloc) {
            Some(url) => format!("<a href=\"{}\">{}</a>", url, trimmed_file),
            None => trimmed_file.to_string(),
        }
    }

    /// Always get correct markdown for a CodeLoc.
    pub fn html_code_url(&self, code: &CodeLocSer) -> String {
        let trimmed_file = self.project.settings.trim_base(&code.file);
        let code_fmt = format!("{}[{}]", trimmed_file, code.line);

        match self.format_code_url_maybe(code) {
            Some(url) => format!("<a href=\"{}\">{}</a>", url, code_fmt),
            None => format!("{}", code_fmt),
        }
    }

    /// Format the url, depending on whether there is a url_fmt available.
    ///
    /// This expects that if url_fmt exists that it MUST be valid.
    pub fn format_code_url_maybe(&self, code: &CodeLocSer) -> Option<String> {
        match self.settings.code_url {
            Some(ref ufmt) => Some(expect!(self.format_code_url(ufmt, code))),
            None => None,
        }
    }

    pub fn format_code_url(&self, url_fmt: &str, code: &CodeLocSer) -> Result<String, String> {
        let file = self.project.settings.trim_base(&code.file);
        strfmt_code_url(url_fmt, file, code.line)
    }

    fn tag_details_begin(&self, w: &mut dyn io::Write, summary: &str) -> io::Result<()> {
        write!(w, "<details>\n<summary><b>{}</b></summary>\n\n", summary)?;
        Ok(())
    }

    fn tag_details_end(&self, w: &mut dyn io::Write) -> io::Result<()> {
        write!(w, "</details>\n\n")?;
        Ok(())
    }
}

pub fn strfmt_code_url(url_fmt: &str, file: &str, line: u64) -> Result<String, String> {
    let l = (line + 1).to_string();
    let vars = hashmap! {
        "file".to_string() => file,
        "line".to_string() => &l,
    };
    ::strfmt::strfmt(url_fmt, &vars)
        .map_err(|e| format!("error formatting url={} error={}", url_fmt, e.to_string()))
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
