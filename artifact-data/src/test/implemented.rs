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

use test::dev_prelude::*;
use name::{self, Name, Type, SubName};
use implemented::parse_locations;

pub fn replace_links(raw: &str) -> String {
    raw.replace('%', "#")
}

#[test]
fn sanity_parse_locations() {
    let example = r#"
This is some kind of text file.
There are links to code like below
%SPC-example %tst-example %TsT-ExAmPle

Some are not on the beginning of the line: %SPC-right
Some have a period after them like %SPC-period.
Multi: %SPC-one %SPC-two%SPC-three
Repeat: %SPC-repeat %SPC-repeat
Not valid: %REQ-foo
Also not valid: %REQ-.foo
Also not valid: %SPC--.foo
Also not valid: %SPC
Also not valid: %TST

Some are legitamate subnames:
%SPC-sub.name

And to the right:
    %SPC-right.sub
"#;
    let expected = &[
        (3, name!("SPC-example"), None),
        (3, name!("TST-example"), None),
        (3, name!("TST-example"), None),
        (5, name!("SPC-right"), None),
        (6, name!("SPC-period"), None),
        (7, name!("SPC-one"), None),
        (7, name!("SPC-two"), None),
        (7, name!("SPC-three"), None),
        (8, name!("SPC-repeat"), None),
        (8, name!("SPC-repeat"), None),
        (16, name!("SPC-sub"), Some(subname!(".name"))),
        (19, name!("SPC-right"), Some(subname!(".sub"))),
    ];
    let locations = parse_locations(replace_links(example).as_bytes()).unwrap();

    assert_eq!(expected, locations.as_slice());
}


