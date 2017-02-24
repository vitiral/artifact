//! #TST-cmd-tutorial

use dev_prefix::*;
use core::prefix::*;
use core::load;
use cmd::tutorial as tut;

lazy_static! {
    pub static ref CWD: PathBuf = env::current_dir().unwrap();
    pub static ref TEST_DIR: PathBuf = CWD.join(PathBuf::from(
        file!()).parent().unwrap().to_path_buf());
}

#[test]
/// just test some assumptions, like that the different "levels"
/// of files aren't equal and that toml files can be loaded
/// by artifact
fn test_tutorial_basic() {
    assert_ne!(tut::D_SETTINGS_1_TOML, tut::D_SETTINGS_2_TOML);
    assert_ne!(tut::D_SETTINGS_1_TOML, tut::D_SETTINGS_4_TOML);
    assert_ne!(tut::D_SETTINGS_2_TOML, tut::D_SETTINGS_4_TOML);

    assert_ne!(tut::D_LOAD_1_PY, tut::D_LOAD_2_PY);
    assert_ne!(tut::D_LOAD_1_TOML, tut::D_LOAD_2_TOML);

    let toml_files = vec![tut::D_TUTORIAL_TOML,
                          tut::D_HIGH_LEVEL_TOML,
                          tut::D_PURPOSE_TOML,
                          tut::D_LOAD_1_TOML,
                          tut::D_LOAD_2_TOML];

    let settings_files =
        vec![tut::D_SETTINGS_1_TOML, tut::D_SETTINGS_2_TOML, tut::D_SETTINGS_4_TOML];

    let p = Path::new("foo");
    for (i, toml) in toml_files.iter().enumerate() {
        let mut project = Project::default();
        let text = str::from_utf8(toml).unwrap();
        load::load_toml(&p, text, &mut project)
            .expect(&format!("could not load tutorial toml at index: {}", i));
    }

    for (i, toml) in settings_files.iter().enumerate() {
        let text = str::from_utf8(toml).unwrap();
        let tbl = load::parse_toml(text).unwrap();
        Settings::from_table(&tbl)
            .expect(&format!("could not load tutorial settings at index: {}", i));
    }
}

#[test]
fn test_line_length() {
    let files = vec![("tut_toml", tut::D_TUTORIAL_TOML),
                     ("tut_md", tut::D_TUTORIAL_MD),

                     ("capitols.csv", tut::D_CAPITOLS_CSV),
                     //("flash-cards.htm", tut::D_FLASH_CARD_CHALLENGE_HTM), # htm exempt
                     ("high_lvl.toml", tut::D_HIGH_LEVEL_TOML),
                     ("purpose.toml", tut::D_PURPOSE_TOML),

                     ("load-1.py", tut::D_LOAD_1_PY),
                     ("load_-1.toml", tut::D_LOAD_1_TOML),
                     ("load-2.py", tut::D_LOAD_2_PY),
                     ("load_-2.toml", tut::D_LOAD_2_TOML),

                     ("test_load.py", tut::D_TEST_LOAD_PY),
                     ("test_data.csv", tut::D_TEST_DATA_CSV),
                     ("set 1", tut::D_SETTINGS_1_TOML),
                     ("set 2", tut::D_SETTINGS_2_TOML),
                     ("set 4", tut::D_SETTINGS_4_TOML)];

    for (fname, f) in files {
        let s = str::from_utf8(f).unwrap();
        let mut long = vec![];
        for (i, l) in s.replace("\r", "").split('\n').enumerate() {
            if l.len() > 80 {
                long.push((i, l.len()));
            }
        }
        if !long.is_empty() {
            panic!("{}: several (line,len) have len > 80: {:?}", fname, long);
        }
    }
}

#[test]
/// just make sure we can run the tutorial without errors
/// in any order
fn test_run_through() {
    let tmp = TEST_DIR.join("test_tmp");
    fs::create_dir(&tmp).unwrap();
    tut::run_cmd(&tmp, 1).expect("part 1");
    tut::run_cmd(&tmp, 2).expect("part 2");
    tut::run_cmd(&tmp, 3).expect("part 3");
    tut::run_cmd(&tmp, 4).expect("part 4");
    tut::run_cmd(&tmp, 5).expect("part 5");

    tut::run_cmd(&tmp, 1).expect("part 1");
    tut::run_cmd(&tmp, 4).expect("part 4");
    tut::run_cmd(&tmp, 3).expect("part 3");
    tut::run_cmd(&tmp, 5).expect("part 5");
    tut::run_cmd(&tmp, 2).expect("part 2");

    tut::run_cmd(&tmp, 5).expect("part 5");
    tut::run_cmd(&tmp, 4).expect("part 4");

    fs::remove_dir_all(&tmp).expect("couldn't remove");
}
