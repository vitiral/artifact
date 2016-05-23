#![allow(dead_code, unused_imports, unused_variables)]

use std::path::PathBuf;

mod test_load;

lazy_static!{
    pub static ref TEST_DIR: PathBuf = PathBuf::from(file!()).parent().unwrap().to_path_buf();
    pub static ref TDATA_DIR: PathBuf = TEST_DIR.join(PathBuf::from("data"));
    pub static ref TEMPTY_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("empty"));
    pub static ref TSIMPLE_DIR: PathBuf = TDATA_DIR.join(PathBuf::from("simple"));
}
