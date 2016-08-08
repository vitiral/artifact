
use super::super::data;


#[test]
fn test_line_length() {
    let files = vec![
        ("capitols", data::capitols_csv::DATA),
        // ("html", data::exercise_htm::DATA),  // html is exempt from this rule
        ("set 1", data::settings_1::DATA),
        ("set 2", data::settings_2::DATA),
        ("set 3", data::settings_4::DATA),
        ("tst_load.py", data::test_load_py::DATA),
        ("tut_md", data::tutorial_md::DATA),
        ("tut_rsk", data::tutorial_rsk::DATA)];

    for (fname, f) in files {
        for (i, l) in f.split('\n').enumerate() {
            assert!(l.len() <= 80, "{}: line {} has len > 80: {}", fname, i, l.len())
        }
    }
}
