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
use nav;
use graph;

pub(crate) fn view_artifact(model: &Model, name: &Name) -> HtmlApp {
    let page = match model.shared.artifacts.get(name) {
        Some(ref art) => view_existing_artifact(model, art),
        None => {
            return html![
            <h3 class=H3,>{format!("Artifact with name {:?} not found", name)}</h3>
        ]
        }
    };

    nav::view_nav(model, page)
}

fn view_existing_artifact(model: &Model, art: &ArtifactSer) -> HtmlApp {
    html! [
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
        { art.impl_.html() }
        <div><textarea readonly=true, cols=80, rows=100,>
            { &art.text }
        </textarea></div>
    ]
}

impl CompletedExt for Completed {
    fn spc_html(&self) -> HtmlApp {
        let color = match self.spc_points() {
            0 => RED,
            1 => ORANGE,
            2 => BLUE,
            3 => OLIVE,
            _ => unreachable!(),
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
            _ => unreachable!(),
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
            _ => unreachable!(),
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
