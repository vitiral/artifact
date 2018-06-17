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
use view;

pub(crate) fn view_edit(model: &Model, id: usize) -> ViewResult {
    ViewResult {
        page: view_edit_page(model, id),
        nav_extra: None,
    }
}


pub(crate) fn handle_edit_artifact(model: &mut Model, id: usize, field: Field) {
    let artifact = match model.editing.get_mut(&id) {
        Some(a) => a,
        None => panic!("TODO: got invalid editing artifact"),
    };
    match field {
        Field::Name(v) => artifact.name = v,
        Field::File(v) => artifact.file = v,
        Field::Done(v) => artifact.done = v,
        Field::Text(v) => artifact.text = v,
        Field::Partof(index, op) => match op {
            FieldOp::Create => artifact.partof.push("".into()),
            FieldOp::Update(v) => artifact.partof[index] = v,
            FieldOp::Delete => {
                artifact.partof.remove(index);
            }
        },
    }
}

pub(crate) fn handle_start_edit(model: &mut Model, id: usize, ty: &StartEditType) {
    let artifact = match *ty {
        StartEditType::New => ArtifactEdit::default(),
        StartEditType::Current => {
            if let View::Artifact(ref name) = model.view {
                let art = expect!(
                    model.shared.artifacts.get(name),
                    "{} viewed but not here",
                    name,
                );
                ArtifactEdit::from_artifact(art)
            } else {
                panic!("wrong view");
            }
        }
    };
    model.editing.insert(id, artifact);
}

fn view_edit_page(model: &Model, id: usize) -> HtmlApp {
    let art = match model.editing.get(&id) {
        Some(a) => a,
        None => {
            return html![
                <div>{ "Editing artifact not found"}</div>
            ]
        }
    };

    html![
        <div>
            <span><button
                class=(BTN, ACE_WHITE, ACE_BG_BLACK),
                id="edit-save",
                onclick=|_| Msg::SendUpdate(vec![id]),
                title="save",
            >
                { fa_icon(FA_SAVE) }
                <span class=ML1,>{ "Save" }</span>
            </button></span>

            <span><button
                class=(BTN, ACE_WHITE, ACE_BG_BLACK),
                id="edit-cancel",
                onclick=|_| Msg::StopEdit(id),
                title="cancel edit",
            >
                { fa_icon(FA_TRASH) }
                <span class=ML1,>{ "Cancel" }</span>
            </button></span>
        </div>

        // NAME
        // TODO: to the right of name/partof put a "relationship" graph that dynamically updates
        <div><h1 class=H1,>
            <label class=MR2,>{ "Editing:" }</label>
            <input id="edit-name",
                type="text",
                class=(H1, FIELD),
                value=art.name.to_string(),
                oninput=|e| Msg::EditArtifact(id, Field::Name(e.value)),
            >
            </input>

        </h1></div>

        // PARTOF
        <div>
            <div>
                <span class=(BOLD),>{ "Partof:" }</span>
                <button
                    id="create-partof",
                    class=(BTN),
                    onclick=|_| Msg::EditArtifact(
                       id, Field::Partof(0, FieldOp::Create)
                    ),
                    title="create",
                >
                    { fa_icon(FA_PLUS_SQUARE) }
                </button>
            </div>
            { view_partof(model, id, art) }
        </div>

        <div class=(MY1),>
            <label class=(BOLD, MR1),>{ "File:" }</label>
            <input id="edit-file",
             type="text",
             class=(FIELD),
             value=art.file.to_string(),
             oninput=|e| Msg::EditArtifact(id, Field::File(e.value)),
            >
            </input>
        </div>

        <div class=(MY1),>
            <label class=(BOLD, MR1),>{ "Done:" }</label>
            <input id="edit-done",
             type="text",
             class=(FIELD),
             value=art.done.to_string(),
             oninput=|e| Msg::EditArtifact(id, Field::Done(e.value)),
            >
            </input>
        </div>

        <div class=(BOLD, MT1),>{ "Text:" }</div>
        <div class=(CLEARFIX, PY1),>
            <div class=(SM_COL, SM_COL_12, MD_COL_6, LG_COL_6),>
                <textarea id="edit-text",
                    value=art.text.to_string(),
                    oninput=|e| Msg::EditArtifact(id, Field::Text(e.value)),
                    class=TEXTAREA,
                    rows=50,
                >
                </textarea>
            </div>

            <div class=(SM_COL, SM_COL_12, MD_COL_6, LG_COL_6),>
                { view::markdown_html(model, &art.name, &art.text) }
            </div>
        </div>
    ]
}

fn view_partof(_: &Model, id: usize, artifact: &ArtifactEdit) -> HtmlApp {
    let view_part = |(index, name): (usize, &String)| {
        let id_str = format!("edit-partof-{}", index);
        html![
        <div>
            <button
                id=format!("rm-partof-{}", index),
                class=(BTN),
                onclick=|_| Msg::EditArtifact(
                   id, Field::Partof(index, FieldOp::Delete)
                ),
                title="remove",
            >
                { fa_icon(FA_TIMES) }
            </button>

            <input id=id_str.to_owned(),
                name=id_str,
                type="text",
                class=(FIELD),
                value=name.to_owned(),
                oninput=|e| Msg::EditArtifact(
                    id, Field::Partof(index, FieldOp::Update(e.value))
                ),
            >
            </input>
        </div>
    ]
    };
    html![<div>
            { for artifact.partof.iter().enumerate().map(view_part) }
    </div>]
}
