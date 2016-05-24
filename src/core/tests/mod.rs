#![allow(dead_code, unused_imports, unused_variables)]

use std::ascii::AsciiExt;
use std::fs;
use std::clone::Clone;
use std::path::{Path, PathBuf};
use std::convert::AsRef;
use std::collections::{HashMap, HashSet, VecDeque};

// Traits
use std::io::{Read, Write};
use std::fmt::Write as WriteStr;
use std::iter::FromIterator;

use toml::{Parser, Value, Table};

mod test_load;
mod test_vars;
mod test_link;

// Data and helpers

lazy_static!{
    pub static ref TEST_DIR: PathBuf = PathBuf::from(file!()).parent().unwrap().to_path_buf();
    pub static ref TDATA_DIR: PathBuf = TEST_DIR.join(PathBuf::from("data"));
    pub static ref TEMPTY_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("empty"));
    pub static ref TSIMPLE_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("simple"));
}

// valid toml, not necessarily all valid artifacts
pub static TOML_GOOD: &'static str = "
[settings]
disabled = false
paths = ['{cwd}/test', '{repo}/test']
repo_names = ['.test']

[REQ-foo]
disabled = false
[SPC-foo]
refs = [1, 2]
[RSK-foo]
[TST-foo]
[REQ-bar]
text = 'bar'
disabled = false
refs = [\"hello\", \"ref\"]
";

// valid rsk file
pub static TOML_RSK: &'static str = "
[settings]
disabled = false
paths = ['{cwd}/data/empty']
repo_names = ['.test']

[REQ-foo]
disabled = false
[SPC-foo]
refs = ['1', '2']
[RSK-foo]
[TST-foo]
[REQ-bar]
disabled = false
partof = 'REQ-[foo, bar-[1,2]], TST-foo'
refs = [\"hello\", \"ref\"]
text = 'bar'
loc = 'LOC-foo: {core}/foo.rs'

[REQ-parts-p1-a]
[REQ-parts-p1-b]

[REQ-parts-p2]
partof = 'REQ-parts-p1'
";

pub static TOML_RSK2: &'static str = "
[settings]
paths = ['test/path']
repo_names = ['.tst']
[REQ-baz]
[RSK-foo-2]
[TST-foo-2]
";

pub static TOML_BAD: &'static str = "[REQ-bad]\nrefs = 'REQ-foo'";  // invalid type
pub static TOML_OVERLAP: &'static str = "[REQ-foo]\n";

pub fn parse_text(t: &str) -> Table {
    Parser::new(t).parse().unwrap()
}

pub fn get_table<'a>(tbl: &'a Table, attr: &str) -> &'a Table {
    match tbl.get(attr).unwrap() {
        &Value::Table(ref t) => t,
        _ => unreachable!()
    }
}
