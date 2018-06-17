/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2017  Garrett Berg <@vitiral, vitiral@gmail.com>
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
//! This module defines all operations around testing artifact names

use ergo::json;

use super::dev_prelude::*;

// HELPERS and TRAITS

// this purposely doesn't use the definition from `name.rs`
const GEN_NAME_RE: &str = r#"(?x)
(REQ|SPC|TST)-              # the type followed by `-`
([a-zA-Z0-9_]{1,7}-){0,3}     # an optional number of `elem-` elements
[a-zA-Z0-9_]{1,7}             # required final element
"#;

// lazy_static!{
//     static ref GEN_NAME_PROP: Arc<prop::string::RegexGeneratorStrategy<String>> =
//         Arc::new(prop::string::string_regex(GEN_NAME_RE).unwrap());
// }

#[inline(always)]
pub fn arb_name_string() -> BoxedStrategy<String> {
    GEN_NAME_RE.prop_map(|s| s.to_string()).boxed()
}

#[inline(always)]
pub fn arb_name() -> BoxedStrategy<Name> {
    arb_name_string().prop_map(|n| name!(n)).boxed()
}

/// Return a vector of the `raw` names
pub fn names_raw(names: &[Name]) -> Vec<String> {
    names.iter().map(|n| n.raw.clone()).collect()
}

/// Assert that the name is valid
pub fn assert_names_valid(raw: &[&str]) {
    let errors = raw.iter()
        .map(|r| (*r, Name::from_str(r)))
        .filter_map(|(raw, result)| match result {
            Ok(name) => if raw == name.raw {
                None
            } else {
                panic!("raw was different: {} => {}", raw, name.raw);
            },
            Err(_) => Some(raw),
        })
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!("The following names were not valid:\n{:#?}", errors);
    }
}

/// Assert that the name is valid
pub fn assert_names_invalid(raw: &[&str]) {
    let errors = raw.iter()
        .map(|r| (r, Name::from_str(r)))
        .filter_map(|(raw, result)| match result {
            Ok(_) => Some(raw),
            Err(_) => None,
        })
        .collect::<Vec<_>>();
    if !errors.is_empty() {
        panic!(
            "The following names were valid but shouldn't have been:\n{:#?}",
            errors
        );
    }
}
