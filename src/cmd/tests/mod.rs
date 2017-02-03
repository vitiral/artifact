/// test cmdline modules

use dev_prefix::*;
use std::sync;

mod test_ls;
mod test_tutorial;
mod test_fmt;

lazy_static!{
    pub static ref CWD: PathBuf = env::current_dir().unwrap();
    pub static ref TEST_DIR: PathBuf = CWD.join(PathBuf::from(
        file!()).parent().unwrap().to_path_buf());
    pub static ref TDATA_DIR: PathBuf = TEST_DIR.join(PathBuf::from("data"));
    // TSIMPLE has to be kept behind a lock because test_fmt actually
    // writes to it!
    pub static ref TSIMPLE_DIR: sync::Mutex<PathBuf> =
        sync::Mutex::new(TDATA_DIR.join(PathBuf::from("simple")));
}
