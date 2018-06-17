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

pub(crate) fn view_nav(model: &Model, view: ViewResult) -> HtmlApp {
    let search = &model.nav.search;
    let icon_search = if search.on {
        FA_SEARCH_MINUS
    } else {
        FA_SEARCH
    };
    let router1 = model.router.clone();
    let router2 = model.router.clone();

    html![<div>
        // Top Nav Bar (buttons)
        <div class=(CLEARFIX, MB2, ACE_WHITE, ACE_BG_BLACK, P1),
         style="position: fixed; top: 0; width: 100%;",
        >
            <button class=(BTN, REGULAR), id="search",
             onclick=|_| Msg::ToggleSearch,
             title="Search for an artifact.",>
                { fa_icon(icon_search) }
                <span class=ML1,>{ "Search" }</span>
            </button>

            <button class=(BTN, REGULAR), id="editing",
             onclick=|_| Msg::ToggleEditing,
             title="View artifacts being edited.",>
                { fa_icon(FA_EDIT) }
                <span class=ML1,>{ "Editing" }</span>
            </button>

            <button class=(BTN, REGULAR), id="create",
             onclick=|_| {
                 let id = new_id();
                 Msg::Batch(vec![
                     Msg::StartEdit(id, StartEditType::New),
                     router1.push_hash(Some(&hash_edit(id))),
                 ])
             },
             title="create new artifact",
            >
                { fa_icon(FA_PLUS_SQUARE) }
                <span class=ML1,>{ "Create" }</span>
            </button>

            <button class=(BTN, REGULAR), id="sync",
             onclick=|_| Msg::FetchProject { reload: true },
             title="Sync frontend with file system.",>
                { fa_icon(FA_SYNC) }
                <span class=ML1,>{ "Sync" }</span>
            </button>

            <button class=(BTN, REGULAR), id="graph",
             onclick=|_| {
                router2.push_hash(Some(HASH_GRAPH))
             },
             title="View Graph",
             href="#graph",
             >
                { fa_icon(FA_GRAPH) }
                <span class=ML1,>{ "Graph" }</span>
            </button>

            <button class=(BTN, REGULAR), id="TEST",
             onclick=|_| {
                Msg::PushLogs(vec![
                    Log::error(format!(
                        "<span>Created error at: {}</span>",
                        ::stdweb::web::Date::now(),
                    )),
                ])
             },
             title="TESTING",
             >
                <span>{ "TEST" }</span>
            </button>

            { view.nav_extra.unwrap_or_else(|| html![<>{ "|" }</>]) }

        </div>

        // Embed the pages
        <div class=(CLEARFIX, PY1),
         style="margin-top: 4em;",
        >
            <div class=(SM_COL, SM_COL_6, MD_COL_4, LG_COL_2),>
                // viewing panes
                { error_pane(model) }
                { editing_pane(model) }
                { search_pane(model) }
            </div>

            <div class=(SM_COL, SM_COL_11, MD_COL_7, LG_COL_9),>
                // rest of page
                { view.page }
            </div>
        </div>
    </div>]
}

fn error_pane(model: &Model) -> HtmlApp {
    if model.logs.error.is_empty() {
        return html![<span></span>];
    }

    fn error_log(id: usize, log: &Log) -> HtmlApp {
        let msg = match Node::from_html(&log.html) {
            Ok(node) => VNode::VRef(node),
            Err(err) => html![
                <b>{ format!(
                    "INTERNALERROR: invalid html: {:?}\nERROR:{:?}",
                    log.html,
                    err)
                }</b>
            ],
        };
        html![
            <button class=(BTN, REGULAR),
             id=format!("close-err-{}", id),
             onclick=|_| Msg::ClearLogs(ClearLogs::Error(vec![id])),
             title="clear error",
            >
                { fa_icon(FA_TIMES) }
                <span class=(ML1, ACE_RED),>
                    { msg }
                </span>
            </button>
        ]
    }

    html![<div class=(BORDER, MR1),>
        <div class=H3,>
            <button class=BTN,
             id="close-err-all",
             onclick=|_| Msg::ClearLogs(ClearLogs::ErrorAll),
            >
                { fa_icon(FA_TIMES) }
                <span class=(ML1, ACE_RED),>
                    { "Clear All Errors" }
                </span>
            </button>
        </div>
        { for model.logs.error.iter().map(|(id, log)| error_log(*id, log)) }
    </div>]
}

fn editing_pane(model: &Model) -> HtmlApp {
    if !model.nav.editing.on {
        return html![<div></div>];
    }

    fn editing_name_html(id: usize, name: &str) -> HtmlApp {
        let name = if name.is_empty() {
            "NOT YET NAMED"
        } else {
            name
        };
        html![<div>
            <a href=format!("#edit/{}", id), class=BTN,>
                { name }
            </a>
        </div>]
    }

    let names = match parse_regex(&model.nav.editing.value) {
        Ok(re) => html![<div>
            { for model.editing
                .iter()
                .filter(|(_, a)| re.is_match(&a.name))
                .map(|(id, art)| editing_name_html(*id, &art.name))
            }
        </div>],
        Err(err) => err,
    };

    html![<div class=(BORDER, MR1),>
        <div><h2 class=H2,>
            { "Editing" }
        </h2></div>
        <input id="editing-search",
         value=model.nav.editing.value.clone(),
         oninput=|e| Msg::SetNavEditing(e.value),
         class=INPUT,
         ></input>
        { names }
    </div>]
}

fn search_pane(model: &Model) -> HtmlApp {
    if !model.nav.search.on {
        return html![<div></div>];
    }

    let names = match parse_regex(&model.nav.search.value) {
        Ok(re) => html![<div>
            { for model.shared.artifacts
                .keys()
                .filter(|n| re.is_match(n.as_str()))
                .map(|n| name::name_html(model, n))
            }
        </div>],
        Err(err) => err,
    };

    html![<div class=(BORDER, MR1),>
        <h2 class=H2,>{ "Search" }</h2>
        <input
         id="search-input",
         value=model.nav.search.value.clone(),
         oninput=|e| Msg::SetNavSearch(e.value),
         class=INPUT,
         ></input>
        { names }
    </div>]
}
