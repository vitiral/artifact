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

// Data and helpers

lazy_static!{
    pub static ref CWD: PathBuf = env::current_dir().unwrap();
    pub static ref TEST_DIR: PathBuf = CWD.join(PathBuf::from(
        file!()).parent().unwrap().to_path_buf());
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
loc = 'LOC-foo'
refs = ['1', '2']
partof = 'REQ-foo'
[LOC-foo]
[RSK-foo]
[TST-foo]
partof = 'SPC-dne'
loc = 'LOC-tst-foo'
[LOC-tst-foo]
[SPC-bar]
disabled = false
partof = 'REQ-[foo, bar-[1,2]]'
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

// tests specifically made for linking tests
pub static TOML_LINK: &'static str = "
[REQ-core]

# bob
[REQ-core-bob]

[SPC-core-bob]
partof = 'REQ-core-bob'

# bob 1 (done, partially tested)
[SPC-core-bob-1]
loc = 'LOC-core-bob-1'

[TST-core-bob-1]
partof = 'SPC-core-bob-1'
[TST-core-bob-1-a]
loc = 'LOC-tst-core-bob-1-a'
[TST-core-bob-1-b]
[TST-core-bob-1-b-1]
[TST-core-bob-1-b-2]
loc = 'LOC-tst-core-bob-1-b-2'

[LOC-core-bob-1]
[LOC-tst-core-bob-1-a]
[LOC-tst-core-bob-1-b-2]

# bob 2 (not done)
[SPC-core-bob-2]
[SPC-core-bob-2-a]
[SPC-core-bob-2-b]
loc = 'LOC-core-bob-2-b'

[TST-core-bob-2-a] # tested but not implemented, possible in TDD
partof = 'SPC-core-bob-2-a'
loc = 'LOC-tst-core-bob-2-a'
[TST-core-bob-2-b] # implemented but not tested
partof = 'SPC-core-bob-2-b'

[LOC-core-bob-2-b]
[LOC-tst-core-bob-2-a]

# joe and jane, only requirements
[REQ-core-joe]
[REQ-core-jane]

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
