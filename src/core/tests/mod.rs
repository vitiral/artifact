#![allow(dead_code, unused_imports, unused_variables)]

use std::env;
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
mod test_core;

// Data and helpers

lazy_static!{
    pub static ref CWD: PathBuf = env::current_dir().unwrap();
    pub static ref TEST_DIR: PathBuf = CWD.join(PathBuf::from(
        file!()).parent().unwrap().to_path_buf());
    pub static ref TDATA_DIR: PathBuf = TEST_DIR.join(PathBuf::from("data"));
    pub static ref TEMPTY_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("empty"));
    pub static ref TSIMPLE_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("simple"));
    pub static ref TINVALID_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("invalid"));
    pub static ref TLOC_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("loc"));
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
loc = '{core}/foo.rs'
refs = ['1', '2']
partof = 'REQ-foo'
[RSK-foo]
[TST-foo]
partof = 'SPC-dne'

[SPC-bar]
loc = '{core}/foo.rs'
disabled = false
partof = 'REQ-[foo, bar-[1,2]]'
refs = [\"hello\", \"ref\"]
text = 'bar'

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

// tests specifically made for linking tests
pub static TOML_LINK: &'static str = "
[REQ-core]

# bob
[REQ-core-bob]

[SPC-core-bob]
partof = 'REQ-core-bob'

# bob 1 (done, partially tested)
[SPC-core-bob-1]
# loc

[TST-core-bob-1]
partof = 'SPC-core-bob-1'
[TST-core-bob-1-a]
# loc
[TST-core-bob-1-b]
[TST-core-bob-1-b-1]
[TST-core-bob-1-b-2]
# loc

# bob 2 (not done)
[SPC-core-bob-2]
[SPC-core-bob-2-a]
[SPC-core-bob-2-b]
# loc

[TST-core-bob-2-a] # tested but not implemented, possible in TDD
partof = 'SPC-core-bob-2-a'
# loc
[TST-core-bob-2-b] # implemented but not tested
partof = 'SPC-core-bob-2-b'

# joe and jane, only requirements
[REQ-core-joe]
[REQ-core-jane]

";


pub static TOML_OVERLAP: &'static str = "[REQ-foo]\n";
pub static TOML_BAD: &'static str = "[REQ-bad]\nrefs = 'REQ-foo'";
pub static TOML_BAD2: &'static str = "[REQ-bad]\nrefs = 'REQ-foo'";
pub static TOML_BAD_ATTR1: &'static str = "[REQ-foo]\nref=['invalid']\n";
pub static TOML_BAD_ATTR2: &'static str = "[REQ-bad]\nrefs = ['REQ-foo', 2, 'hi']";
pub static TOML_BAD_JSON: &'static str = "{\"REQ-foo\": {\"refs\": [\"hi\"]}}";
pub static TOML_BAD_NAMES1: &'static str = "[REQ-bad]\n[REQ-bad]";
pub static TOML_BAD_NAMES2: &'static str = "[REQ-bad]\nrefs=['hi']\nrefs=['you']";

pub fn parse_text(t: &str) -> Table {
    Parser::new(t).parse().unwrap()
}

pub fn get_table<'a>(tbl: &'a Table, attr: &str) -> &'a Table {
    match tbl.get(attr).unwrap() {
        &Value::Table(ref t) => t,
        _ => unreachable!()
    }
}
