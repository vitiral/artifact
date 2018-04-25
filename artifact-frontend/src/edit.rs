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

// pub(crate) fn update(model: &mut Model, id: usize, field: Field) -> ShouldRender {
//     match field {
//         Field::Name(v) => model.name = v,
//         // StrField::File => model.file = value,
//         Field::Done(v) => model.done = v,
//         Field::Text(v) => model.text = v,
//
//         Field::Partof(id, part) => {
//             unimplemented!()
//         }
//     }
//     true
// }

pub(crate) fn view_edit(model: &Model, id: usize) -> HtmlApp {
    nav::view_nav(model, view_edit_page(model, id))
}

fn view_edit_page(model: &Model, id: usize) -> HtmlApp {
    let art = match model.editing.get(&id) {
        Some(a) => a,
        None => {
            return html![
            <div>{ "Editing artifact not found"} </div>
        ]
        }
    };

    html![
        <h1 class=H1,>
            <span class=MR2,>{ "Editing" }</span>
            <input id="edit-name",
                value=art.name.to_string(),
                oninput=move |e: InputData| Msg::EditArtifact(id, Field::Name(e.value)),
                class=(H1),
                cols=80,
            >
            </input>
        </h1>

        <div class=H3,>
            <span class=(H3, MR2),>{ "File:" }</span>
            <input id="edit-file",
                class=(H3),
                value=art.file.to_string(),
                oninput=move |e: InputData| Msg::EditArtifact(id, Field::File(e.value)),
                cols=80,
            >
            </input>
        </div>

        <div class=H3,>
            <span class=MR2,>{ "Done:" }</span>
            <input id="edit-done",
                value=art.done.to_string(),
                oninput=move |e: InputData| Msg::EditArtifact(id, Field::Done(e.value)),
                cols=80,
            >
            </input>
        </div>

        <div class=(H2),>
            <span class=(H2),>{ "Partof:" }</span>
            <button
                id=format!("create-partof"),
                class=(BTN, H2),
                onclick=move |_| Msg::EditArtifact(
                   id, Field::Partof(0, FieldOp::Create)
                ),
                title="create",
            >
                { fa_icon(FA_PLUS_SQUARE) }
            </button>
        </div>
        { view_partof(model, id, art) }

        <div class=H2,>{ "Text:" }</div>
        <div class=H3,>
            <textarea
                id="edit-text",
                value=art.text.to_string(),
                oninput=move |e: InputData| Msg::EditArtifact(id, Field::Text(e.value)),
                cols=80,
                rows=100,
            >
            </textarea>
        </div>
    ]
}


fn view_partof(model: &Model, id: usize, artifact: &ArtifactEdit) -> HtmlApp {
    let view_part = |(index, name): (usize, &String)| {
        html![
        <div class=REGULAR,>
            <button
                id=format!("rm-partof-{}", index),
                class=(BTN, REGULAR),
                onclick=move |_| Msg::EditArtifact(
                   id, Field::Partof(index, FieldOp::Delete)
                ),
                title="remove",
            >
                { fa_icon(FA_TIMES) }
            </button>

            <input
                id=format!("edit-partof-{}", index),
                value=name.to_owned(),
                oninput=move |e: InputData| Msg::EditArtifact(
                    id, Field::Partof(index, FieldOp::Update(e.value))
                ),
                cols=60,
            >
            </input>
        </div>
    ]
    };
    html![<div>
            { for artifact.partof.iter().enumerate().map(view_part) }
    </div>]
}
