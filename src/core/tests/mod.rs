#![allow(dead_code, unused_imports, unused_variables)]

use std::sync;

use toml::{Parser, Value, Table};

use dev_prefix::*;
use core::types::*;

use core::load;


mod test_toml;
mod test_load;
mod test_vars;
mod test_link;
mod test_core;
mod test_save;
mod test_serde;
mod test_security;

// Mutex that can be used for tests that need to be run serially

// Data and helpers

lazy_static!{
    pub static ref CWD: PathBuf = env::current_dir().unwrap();
    pub static ref TEST_DIR: PathBuf = CWD.join(PathBuf::from(
        file!()).parent().unwrap().to_path_buf());
    pub static ref TDATA_DIR: PathBuf = TEST_DIR.join(PathBuf::from("data"));
    pub static ref TEMPTY_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("empty"));
    // TSIMPLE has to be kept behind a lock because test_fmt actually
    // writes to it!
    pub static ref TSIMPLE_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("simple"));
    pub static ref TINVALID_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("invalid"));
    pub static ref TINVALID_BOUNDS: PathBuf = TINVALID_DIR.join("out-bounds");
    pub static ref TLOC_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("loc"));
}

pub fn load_toml_simple(text: &str) -> Artifacts {
    let mut project = Project::default();
    let path = PathBuf::from("test");
    load::load_toml(&path, text, &mut project).unwrap();
    project.artifacts
}

pub static TOML_SETTINGS: &'static str = "
artifact_paths = ['{cwd}/test', '{repo}/test']
code_paths = ['{cwd}/src', '{repo}/src2']
";


// valid toml, not necessarily all valid artifacts
pub static TOML_GOOD: &'static str = "
[REQ-foo]
[SPC-foo]
[RSK-foo]
[TST-foo]
[REQ-bar]
text = 'bar'
";

// valid toml, not necessarily all valid artifacts
pub static TOML_DONE: &'static str = "
[REQ-foo]
[SPC-foo]
done = 'foo'
[RSK-foo]
[TST-foo]
done = 'foo'
[REQ-bar]
text = 'bar'
[SPC-bar]
[TST-bar]
";

// valid artifact file
pub static TOML_RST: &'static str = "
[REQ-foo]
[SPC-foo]
[RSK-foo]
[TST-foo]
partof = 'SPC-dne'

[SPC-bar]
partof = 'REQ-[foo, bar-[1,2]]'
text = 'bar'
done = 'bar is done'

[REQ-parts-p1-a]
[REQ-parts-p1-b]

[REQ-parts-p2]
partof = 'REQ-parts-p1'
";

pub static TOML_RST2: &'static str = "
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

# bob 1 (done, partially tested)
[SPC-core-bob-1]
# loc

[TST-core-bob-1]
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
# loc
[TST-core-bob-2-b] # implemented but not tested

# joe and jane, only requirements
[REQ-core-joe]
[REQ-core-jane]

";

pub static TOML_OVERLAP: &'static str = "[REQ-foo]\n";
pub static TOML_BAD: &'static str = "[REQ-bad]\npartof = ['REQ-foo']";
pub static TOML_BAD2: &'static str = "[REQ-bad]\npartof = 'REQ-foo'";
pub static TOML_BAD_ATTR1: &'static str = "[REQ-foo]\npart='invalid'\n";
pub static TOML_BAD_ATTR2: &'static str = "[REQ-bad]\npartof = ['REQ-foo', 2, 'hi']";
pub static TOML_BAD_JSON: &'static str = "{\"REQ-foo\": {\"partof\": [\"hi\"]}}";
pub static TOML_BAD_NAMES1: &'static str = "[REQ-bad]\n[REQ-bad]";
pub static TOML_BAD_NAMES2: &'static str = "[REQ-bad]\npartof='hi'\npartof='you'";

pub fn parse_text(t: &str) -> Table {
    Parser::new(t).parse().unwrap()
}

pub fn get_table<'a>(tbl: &'a Table, attr: &str) -> &'a Table {
    match tbl.get(attr).unwrap() {
        &Value::Table(ref t) => t,
        _ => unreachable!(),
    }
}
