
use super::super::data;


#[test]
// #TST-tutorial-line
fn test_line_length() {
    let files = vec![
        ("capitols.csv", data::capitols_csv::DATA),
        ("example.csv", data::example_csv::DATA),
        // ("html", data::exercise_htm::DATA),  // html is exempt from this rule
        ("high lvl", data::high_level_toml::DATA),
        ("load.py", data::load_py::DATA),
        ("load_toml", data::load_toml::DATA),
        ("purpose_toml", data::purpose_toml::DATA),
        ("set 1", data::settings_1::DATA),
        ("set 2", data::settings_2::DATA),
        ("set 4", data::settings_4::DATA),
        ("tst_load.py", data::test_load_py::DATA),
        ("tut_md", data::tutorial_md::DATA),
        ("tut_toml", data::tutorial_toml::DATA)];

    for (fname, f) in files {
        for (i, l) in f.split('\n').enumerate() {
            assert!(l.len() <= 80, "{}: line {} has len > 80: {}", fname, i, l.len())
        }
    }
}

