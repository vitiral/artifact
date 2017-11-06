#![allow(dead_code, unused_imports, unused_variables)]

use std::sync;
use toml;

use dev_prefix::*;
use types::*;
use user;


lazy_static!{
    pub static ref CWD: PathBuf = env::current_dir().unwrap();
    pub static ref TDATA_DIR: PathBuf = CWD.join(PathBuf::from(
        file!()).parent().unwrap().to_path_buf());
    pub static ref TEMPTY_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("empty"));
    pub static ref TSIMPLE_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("simple"));
    pub static ref TEXCLUDE_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("exclude"));
    pub static ref TINVALID_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("invalid"));
    pub static ref TINVALID_BOUNDS: PathBuf = TINVALID_DIR.join("out-bounds");
    pub static ref TLOC_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("loc"));
}

pub fn load_toml_simple(text: &str) -> Artifacts {
    let mut project = Project::default();
    let path = PathBuf::from("test.toml");
    user::load_text(&path, text, &mut project).unwrap();
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
[TST-foo]
[REQ-bar]
text = 'bar'
";

// valid toml, not necessarily all valid artifacts
pub static TOML_DONE: &'static str = "
[REQ-foo]
[SPC-foo]
done = 'foo'
[TST-foo]
done = 'foo'
[SPC-bar]
done = 'bar'
";

// valid artifact file
pub static TOML_RST: &'static str = "
[REQ-foo]
[SPC-foo] # loc
[TST-foo] # loc
partof = 'SPC-dne'

[SPC-bar]  # (1.0, 1.0)
partof = 'REQ-[foo, bar-[1,2]]'
text = 'bar'
done = 'bar is done'

[REQ-parts]
[REQ-parts-p1]
[REQ-parts-p1-a]
[REQ-parts-p1-b]

[REQ-parts-p2]
partof = 'REQ-parts-p1'
";

pub static TOML_RST2: &'static str = "
[REQ-baz]
[TST-foo-2]
";

// tests specifically made for linking tests
pub static TOML_LINK: &'static str = "
[REQ-core]
[SPC-core]
[TST-core]

[REQ-core-bob]

[SPC-core-bob]          # la (CALC)

[SPC-core-bob-1]        # la (1.0, 0.75)


[SPC-core-bob-2]        # calc
[SPC-core-bob-2-a]      #     (0.00, 1.00)
[SPC-core-bob-2-b]      # la  (1.00, 0.00)

[TST-core-bob]          # AVG TST-core-bob-[1,2]

[TST-core-bob-1]        # la  (calc, 0.75)
[TST-core-bob-1-a]      # loc (1.00, 1.00)
[TST-core-bob-1-b]      # avg (0.50, 0.50)
[TST-core-bob-1-b-1]    # na  (0.00, 0.00)
[TST-core-bob-1-b-2]    # loc (1.00, 1.00)

[TST-core-bob-2]        # avg (0.50, 0.50)
[TST-core-bob-2-a]      # loc (1.00, 1.00)
[TST-core-bob-2-b]      # na  (0.00, 0.00)

[REQ-core-jane]
[REQ-core-joe]

[REQ-done]
done = \"test\"
[SPC-done]
done = \"test\"

[SPC-done-1]
[SPC-done-2]
";

pub static TOML_UNFMT: &'static str = r#"
[REQ-foo]
partof = "REQ-single"

[REQ-bar]
partof = "REQ-[foo, bar]"

[REQ-baz]
partof = [
    "REQ-foo",
    "REQ-bar",
]

[REQ-faz]
partof = ["REQ-foo", "REQ-bar-[1,2]"]

[SPC-zed]
partof = "REQ-bar, SPC-foo-[1,2]"
"#;

pub static TOML_FMT: &'static str = r#"[REQ-bar]
partof = 'REQ-foo'

[REQ-baz]
partof = [
    'REQ-bar',
    'REQ-foo',
]

[REQ-faz]
partof = [
    'REQ-bar-1',
    'REQ-bar-2',
    'REQ-foo',
]

[REQ-foo]
partof = 'REQ-single'

[SPC-zed]
partof = [
    'REQ-bar',
    'SPC-foo-1',
    'SPC-foo-2',
]
"#;



pub static TOML_OVERLAP: &'static str = "[REQ-foo]\n";
pub static TOML_BAD_ATTR1: &'static str = "[REQ-foo]\npart='invalid'\n";
pub static TOML_BAD_ATTR2: &'static str = "[REQ-bad]\npartof = ['REQ-foo', 2, 'hi']";
pub static TOML_BAD_JSON: &'static str = "{\"REQ-foo\": {\"partof\": [\"hi\"]}}";
pub static TOML_BAD_NAMES1: &'static str = "[REQ-bad]\n[REQ-bad]";
pub static TOML_BAD_NAMES2: &'static str = "[REQ-bad]\npartof='hi'\npartof='you'";
