
use dev_prefix::*;
use core::prefix::*;
use super::super::init_logger_test;
use core::save;
use core::load;
use core::vars;
use core::types;
use core;

use super::*; // data directory constants



#[test]
/// load a project as text and then convert
fn test_save_idempotent() {
    //init_logger_test();
    // load tsimple and process
    let mut original_text = types::ProjectText::default();
    let mut loaded_dirs = HashSet::new();
    original_text.load(TSIMPLE_DIR.as_path(), &mut loaded_dirs)
        .unwrap();
    let original = core::process_project_text(&original_text).unwrap();

    // serialize tsimple like it would be saved
    // and convert back
    let result_text = types::ProjectText::from_project(&original).unwrap();
    let result = core::process_project_text(&result_text).unwrap();

    // make assertions
    original.equal(&result).unwrap();
    assert_ne!(original_text, result_text);
}
