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

pub(crate) fn name_html(model: &Model, name: &Name) -> HtmlApp {
    let color = match model.shared.artifacts.get(name) {
        Some(art) => art.completed.name_color(),
        None => GRAY,
    };

    html![<div><a href=format!("#{}", name), color=color, class=BTN,>
        { name }
    </a></div>]
}

pub(crate) fn name_color(model: &Model, name: &Name) -> &'static str {
    match model.shared.artifacts.get(name) {
        Some(art) => art.completed.name_color(),
        None => GRAY,
    }
}
