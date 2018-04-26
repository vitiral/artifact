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
#![allow(dead_code)]
pub use yew::prelude::*;
pub use yew::services::console::ConsoleService;
pub use yew_simple::FetchTask;
pub use yew::virtual_dom::VNode;
pub use artifact_ser::*;
pub use ergo_std::*;
pub use ergo_config::*;
pub use stdweb::web::{Node, Window};

lazy_static! {
    static ref ATOMIC_ID: AtomicUsize = ATOMIC_USIZE_INIT;
}

pub(crate) fn new_id() -> usize {
    ATOMIC_ID.fetch_add(1, AtomicOrdering::SeqCst)
}

pub(crate) type HtmlApp = Html<Context, Model>;

pub(crate) struct Context {}

// http://basscss.com/

// Types
pub(crate) const H1: &str = "h1";
pub(crate) const H2: &str = "h2";
pub(crate) const H3: &str = "h3";
pub(crate) const BTN: &str = "btn";
pub(crate) const INPUT: &str = "input";
pub(crate) const TEXTAREA: &str = "textarea";
pub(crate) const BLOCK: &str = "block";
pub(crate) const FIELD: &str = "field";

// Styles
pub(crate) const REGULAR: &str = "regular";
pub(crate) const BOLD: &str = "bold";
pub(crate) const ITALIC: &str = "italic";
pub(crate) const BORDER: &str = "border";

// Alignment
pub(crate) const LEFT: &str = "left";
pub(crate) const LEFT_ALIGN: &str = "left-align";
pub(crate) const RIGHT: &str = "right";

// Controlling columns
pub(crate) const CLEARFIX: &str = "clearfix";

// Padding: top/bottom/right/left + x/y axis
pub(crate) const P1: &str = "p1";
pub(crate) const PY1: &str = "py1";
pub(crate) const PX1: &str = "py1";

// margin right/left/top/bottom + x/y axis
pub(crate) const MT1: &str = "mt1";
pub(crate) const MR1: &str = "mr1";
pub(crate) const MB1: &str = "mb1";
pub(crate) const ML1: &str = "ml1";
pub(crate) const MX1: &str = "mx1";
pub(crate) const MY1: &str = "my1";

pub(crate) const MR2: &str = "mr2";
pub(crate) const MB2: &str = "mb2";

// Colors
pub(crate) const ACE_WHITE: &str = "white";
pub(crate) const ACE_GRAY: &str = "gray";
pub(crate) const ACE_BG_BLACK: &str = "bg-black";
pub(crate) const ACE_BG_GRAY: &str = "bg-gray";
pub(crate) const ACE_RED: &str = "red";

pub(crate) const GRAY: &str = "#DCDEE2";
pub(crate) const OLIVE: &str = "#3DA03D";
pub(crate) const BLUE: &str = "#0074D9";
pub(crate) const ORANGE: &str = "#FF851B";
pub(crate) const RED: &str = "#FF4136";
pub(crate) const PURPLE: &str = "#B10DC9";

// Column controls == must add to 12 in view
pub(crate) const COL: &str = "col";
pub(crate) const COL_1: &str = "col-1";
pub(crate) const COL_2: &str = "col-2";
pub(crate) const COL_3: &str = "col-3";
pub(crate) const COL_4: &str = "col-4";
pub(crate) const COL_5: &str = "col-5";
pub(crate) const COL_6: &str = "col-6";
pub(crate) const COL_7: &str = "col-7";
pub(crate) const COL_10: &str = "col-10";

// Responsive Columns
pub(crate) const SM_COL: &str = "sm-col";
pub(crate) const SM_COL_2: &str = "sm-col-2";
pub(crate) const SM_COL_3: &str = "sm-col-3";
pub(crate) const SM_COL_6: &str = "sm-col-6";
pub(crate) const SM_COL_8: &str = "sm-col-8";
pub(crate) const SM_COL_11: &str = "sm-col-11";
pub(crate) const SM_COL_12: &str = "sm-col-12";

pub(crate) const MD_COL_3: &str = "md-col-3";
pub(crate) const MD_COL_4: &str = "md-col-4";
pub(crate) const MD_COL_5: &str = "md-col-5";
pub(crate) const MD_COL_6: &str = "md-col-6";
pub(crate) const MD_COL_7: &str = "md-col-7";
pub(crate) const MD_COL_8: &str = "md-col-8";
pub(crate) const MD_COL_12: &str = "md-col-12";

pub(crate) const LG_COL_2: &str = "lg-col-2";
pub(crate) const LG_COL_3: &str = "lg-col-3";
pub(crate) const LG_COL_4: &str = "lg-col-4";
pub(crate) const LG_COL_5: &str = "lg-col-5";
pub(crate) const LG_COL_6: &str = "lg-col-6";
pub(crate) const LG_COL_7: &str = "lg-col-7";
pub(crate) const LG_COL_8: &str = "lg-col-8";
pub(crate) const LG_COL_9: &str = "lg-col-9";
pub(crate) const LG_COL_10: &str = "lg-col-10";
pub(crate) const LG_COL_12: &str = "lg-col-12";

// Font Awesome
pub(crate) const FA: &str = "fas";
pub(crate) const FA_GRAPH: &str = "fa-code-branch";
pub(crate) const FA_INFO_CIRCLE: &str = "fa-info-circle";
pub(crate) const FA_EDIT: &str = "fa-edit";
pub(crate) const FA_EYE: &str = "fa-eye";
pub(crate) const FA_SAVE: &str = "fa-floppy-o";
pub(crate) const FA_PLUS_SQUARE: &str = "fa-plus-square";
pub(crate) const FA_SEARCH: &str = "fa-search";
pub(crate) const FA_SEARCH_PLUS: &str = "fa-search-plus";
pub(crate) const FA_SEARCH_MINUS: &str = "fa-search-minus";
pub(crate) const FA_EXCLAMATION: &str = "fa-exclamation";
pub(crate) const FA_EXCLAMATION_CIRCLE: &str = "fa-exclamation-circle";
pub(crate) const FA_SYNC: &str = "fa-sync";
pub(crate) const FA_TRASH: &str = "fa-trash";
pub(crate) const FA_TIMES: &str = "fa-times";

// Custom
pub(crate) const ART_INFO: &str = "art-info";
pub(crate) const SELECT_TINY: &str = "select-tiny";

#[derive(Debug, Clone)]
pub(crate) enum View {
    Graph,
    Artifact(Name),
    Edit(usize),
    NotFound,
}

pub(crate) struct Model {
    // TODO: make ProjectResult
    pub(crate) shared: Arc<ProjectSer>,
    pub(crate) view: View,
    pub(crate) router: Arc<::yew_simple::RouterTask<Context, Model>>,
    pub(crate) nav: Nav,
    pub(crate) graph: Graph,
    pub(crate) fetch_task: Option<FetchTask>,
    pub(crate) console: Arc<ConsoleService>,
    pub(crate) logs: Logs,
    pub(crate) window: Window,
    pub(crate) editing: IndexMap<usize, ArtifactEdit>,
}

pub(crate) enum ClearLogs {
    Error(Vec<usize>),
    ErrorAll,
}

pub(crate) enum Msg {
    SetView(View),

    ToggleSearch,
    ToggleEditing,
    SetNavSearch(String),
    SetNavEditing(String),

    SetGraphSearch(String),
    FetchProject,
    SendUpdate(Vec<usize>),
    RecvProject(ProjectSer),

    PushLogs(Vec<Log>),
    ClearLogs(ClearLogs),

    EditArtifact(usize, Field),
    StartEdit(usize, StartEditType),

    Ignore,
    Batch(Vec<Msg>),
}

pub(crate) enum StartEditType {
    New,
    Current,
}

#[derive(Debug, Default)]
pub(crate) struct Logs {
    pub(crate) error: IndexMap<usize, Log>,
    pub(crate) info: IndexMap<usize, Log>,
}

#[derive(Debug, Clone)]
pub(crate) struct Log {
    pub(crate) level: LogLevel,
    pub(crate) html: String,
}

impl Log {
    pub(crate) fn error(html: String) -> Self {
        Log {
            level: LogLevel::Error,
            html: html,
        }
    }

    pub(crate) fn info(html: String) -> Self {
        Log {
            level: LogLevel::Info,
            html: html,
        }
    }
}

#[derive(Debug, Copy, Clone)]
pub(crate) enum LogLevel {
    Error,
    Info,
}

#[derive(Debug, Default, Clone)]
/// Navigation bar
pub(crate) struct Nav {
    pub(crate) search: Search,
    pub(crate) editing: Search,
}

#[derive(Debug, Default, Clone)]
/// Graph View / search
pub(crate) struct Graph {
    pub(crate) search: String,
}

#[derive(Debug, Default, Clone)]
/// Search settings
pub(crate) struct Search {
    pub(crate) on: bool,
    pub(crate) value: String,
}

impl Search {
    pub(crate) fn with_on(self, on: bool) -> Self {
        Self {
            on: on,
            value: self.value,
        }
    }
}

pub(crate) trait CompletedExt {
    fn spc_html(&self) -> HtmlApp;
    fn tst_html(&self) -> HtmlApp;
    fn name_color(&self) -> &'static str;
}

/// Editable Artifact
#[derive(Debug, Default, Clone, PartialEq)]
pub(crate) struct ArtifactEdit {
    pub original_id: Option<HashIm>,
    pub name: String,
    pub file: String,
    pub partof: Vec<String>,
    pub done: String,
    pub text: String,
}

impl ArtifactEdit {
    pub(crate) fn from_artifact(art: &ArtifactSer) -> ArtifactEdit {
        ArtifactEdit {
            original_id: Some(art.id.clone()),
            name: art.name.to_string(),
            file: art.file.clone(),
            partof: art.partof.iter().map(|n| n.to_string()).collect(),
            done: art.impl_
                .as_done()
                .map(String::from)
                .unwrap_or_else(String::new),
            text: art.text.clone(),
        }
    }
}

/// The field that is being edited.
pub enum Field {
    Name(String),
    File(String),
    Done(String),
    Text(String),
    /// Create/Update/Delete partof at index.
    Partof(usize, FieldOp),
}

pub enum FieldOp {
    Create,
    Update(String),
    Delete,
}

/// These are unbelivably annoying to create.
///
/// The FA library _mutates_ the item with it's class assigned, which means that when yew tries to
/// call `parent.remove_child` the child cannot be found... since it changed.
///
/// I'm not totally sure what's happening here but it ain't fun.
///
/// Anyway, the issue can be avoided by wrapping it in an additional element, hence
/// the extra `<span>...</span>`
pub(crate) fn fa_icon(icon: &str) -> HtmlApp {
    let icon = format!(r#"<span><i class="{} {}"></i></span>"#, FA, icon);
    let icon = Node::from_html(icon.trim()).expect("fa-icon");
    VNode::VRef(icon)
}

/// Parse the regex. If it is invalid, return the html error
/// message to display to the user.
pub(crate) fn parse_regex(s: &str) -> Result<Regex, HtmlApp> {
    Regex::new(s).map_err(|e| {
        html![
            <a
             href="https://docs.rs/regex/0.2.10/regex/#syntax",
             title="See syntax definition.",
             class=(RED, BTN, BOLD),
            >
            { "INVALID REGEX" }
            </a>
        ]
    })
}

/// Render the markdown correctly.
pub(crate) fn markdown_html(model: &Model, markdown: &str) -> HtmlApp {
    let value = js!{
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
