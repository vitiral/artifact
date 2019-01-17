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
use graph;
use view;

pub(crate) fn view_artifact(model: &Model, name: &Name) -> ViewResult {
    let page = match model.shared.artifacts.get(name) {
        Some(ref art) => view_existing_artifact(model, art),
        None => html![
            <div><h3 class=H3,>
                {format!("Artifact with name {:?} not found", name)}
            </h3></div>
        ],
    };

    ViewResult {
        page,
        nav_extra: None,
    }
}

fn view_existing_artifact(model: &Model, art: &ArtifactSer) -> HtmlApp {
    let router = model.router.clone();
    html! [
        <div>
            // TODO: do something special if artifact already exists
            <button class=(BTN, ACE_WHITE, ACE_BG_BLACK), id="edit",
             onclick=|_| {
                 let id = new_id();
                 Msg::Batch(vec![
                    Msg::StartEdit(id, StartEditType::Current),
                    router.push_hash(Some(&hash_edit(id))),
                 ])
             },
             title="Edit this artifact.",
            >
                { fa_icon(FA_EDIT) }
                <span class=ML1,>{ "Edit" }</span>
            </button>
        </div>
        <div><h1 class=H1,>{ &art.name }</h1></div>
        { graph::artifact_part_html(model, art) }

        // responive colums for spc% + tst%
        <div class=(CLEARFIX, PY1),>
            <div class=(SM_COL, SM_COL_6, MD_COL_4, LG_COL_2),>
                <span class=(MR1, BOLD),>{ "spc%" }</span>
                { art.completed.spc_html() }
            </div>
            <div class=(SM_COL, SM_COL_6, MD_COL_4, LG_COL_2),>
                <span class=(MR1, BOLD),>{ "tst%" }</span>
                { art.completed.tst_html() }
            </div>
        </div>
        <div>
            <span class=(MR1, BOLD),>{ "file" }</span>
            { &art.file }
        </div>

        { art.impl_.html() }
        { view::markdown_html(model, art.name.as_str(), &art.text) }
    ]
}

impl CompletedExt for Completed {
    fn spc_html(&self) -> HtmlApp {
        let color = match self.spc_points() {
            0 => RED,
            1 => ORANGE,
            2 => BLUE,
            3 => OLIVE,
            _ => panic!("invalid spc_points"),
        };
        html![
            <span color=color, class=BOLD,>{
                format!("{:.1}", self.spc * 100.0)
            }</span>
        ]
    }

    fn tst_html(&self) -> HtmlApp {
        let color = match self.tst_points() {
            0 => RED,
            1 => ORANGE,
            2 => OLIVE,
            _ => panic!("invalid tst_points"),
        };
        html![
            <span class=(color, BOLD),>{
                format!("{:.1}", self.tst * 100.0)
            }</span>
        ]
    }

    fn name_color(&self) -> &'static str {
        match self.spc_points() + self.tst_points() {
            0 => RED,
            1 | 2 => ORANGE,
            3 | 4 => BLUE,
            5 => OLIVE,
            _ => panic!("invalid name points"),
        }
    }
}

trait ImplSerExt {
    fn html(&self) -> HtmlApp;
}

impl ImplSerExt for ImplSer {
    fn html(&self) -> HtmlApp {
        match *self {
            ImplSer::Done(ref d) => html![
                <div>
                    <span class=(BOLD, MR1),>{ "Defined as done:" }</span>
                    <span>{ d }</span>
                </div>
            ],
            ImplSer::Code(ref code) => primary_html(&code.primary),
            ImplSer::NotImpl => html![<span></span>],
        }
    }
}

fn primary_html(primary: &Option<CodeLocSer>) -> HtmlApp {
    if let Some(ref loc) = primary {
        html![<div>
            <span class=(BOLD, MR1),>{ "Implemented:" }</span>
            // TODO: add link
            <span>{ format!("{:?}", loc) }</span>
        </div>]
    } else {
        html![<span></span>]
    }
}
