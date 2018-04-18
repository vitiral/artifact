#![allow(dead_code)]
pub use chrono::prelude::*;
pub use yew::prelude::*;
pub use yew::services::console::ConsoleService;
pub use artifact_lib::*;
pub use ergo_std::*;
pub use ergo_config::*;
pub use path_abs::*;

pub(crate) type HtmlApp = Html<Context, Model>;

pub(crate) struct Context {}

// http://basscss.com/

pub(crate) const H1: &str = "h1";
pub(crate) const H2: &str = "h2";
pub(crate) const H3: &str = "h3";
pub(crate) const BTN: &str = "btn";

pub(crate) const REGULAR: &str = "regular";
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

// Colors
pub(crate) const BG_BLACK: &str = "bg-black";
pub(crate) const BLUE: &str = "blue";
pub(crate) const BOLD: &str = "bold";
pub(crate) const ITALIC: &str = "italic";
pub(crate) const GRAY: &str = "gray";
pub(crate) const OLIVE: &str = "olive";
pub(crate) const RED: &str = "red";
pub(crate) const WHITE: &str = "white";
pub(crate) const YELLOW: &str = "yellow";

// Border for tables
pub(crate) const BORDER: &str = "border";

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

// Font Awesome
pub(crate) const FA: &str = "fa";
pub(crate) const FA_INFO_CIRCLE: &str = "fa-info-circle";
pub(crate) const FA_EYE: &str = "fa-eye";
pub(crate) const FA_FLOPPY_O: &str = "fa-floppy-o";
pub(crate) const FA_PLUS_SQUARE: &str = "fa-plus-square";
pub(crate) const FA_SEARCH: &str = "fa-search";
pub(crate) const FA_EXCLAMATION: &str = "fa-exclamation";
pub(crate) const FA_EXCLAMATION_CIRCLE: &str = "fa-exclamation-circle";
pub(crate) const FA_TRASH: &str = "fa-trash";
pub(crate) const FA_TIMES: &str = "fa-times";

// Cutom
pub(crate) const ART_INFO: &str = "art-info";
pub(crate) const SELECT_TINY: &str = "select-tiny";



#[derive(Debug, Clone)]
pub(crate) enum View {
    List,
    Artifact(Name),
}

pub(crate) struct Model {
    pub(crate) shared: Arc<ProjectSer>,
    pub(crate) view: View,
    pub(crate) router: ::yew_router::RouterTask<Context, Model>,
}

pub(crate) enum Msg {
    SetView(View),
    Ignore,
}
