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

/// Used for inserting into markdown, etc.
pub(crate) fn name_markdown(model: &Model, name: &Name, sub: Option<&SubName>) -> String {
    if let Some(art) = model.shared.artifacts.get(name) {
        if let Some(s) = sub {
            return name_subname_markdown(model, art, name, s);
        } else {
            name_html_raw(
                name.as_str(),
                None,
                name_color(model, name),
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

fn name_subname_markdown(model: &Model, art: &ArtifactSer, name: &Name, sub: &SubName) -> String {
    if art.subnames.contains(sub) {
        name_html_raw(
            name.as_str(),
            Some(sub.as_str()),
            name_color(model, name),
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

pub(crate) fn name_color(model: &Model, name: &Name) -> &'static str {
    match model.shared.artifacts.get(name) {
        Some(art) => art.completed.name_color(),
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
