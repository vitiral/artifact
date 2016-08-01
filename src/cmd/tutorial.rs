
pub use std::path::{PathBuf, Path};
pub use std::fs;
pub use std::io::Read;
pub use std::env;
use super::types::*;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    // #SPC-init
    SubCommand::with_name("tutorial")
        .about("start the interactive tutorial")
        .settings(&[AS::DeriveDisplayOrder, AS::ColoredHelp])
        .arg(Arg::with_name("part")
             .help("the part to set the tutorial at"))
}

lazy_static!{
    // directory paths
}

pub fn do_tutorial(path: &Path) -> io::Result<()> {
    let CWD: PathBuf = env::current_dir().unwrap();
    let RSK_DIR: PathBuf = CWD.join(PathBuf::from(".rsk"));
    let SRC_DIR: PathBuf = CWD.join(PathBuf::from("flash"));
    let TESTS_DIR: PathBuf = SRC_DIR.join(PathBuf::from("tests"));
    let DOCS_DIR: PathBuf = CWD.join(PathBuf::from("docs"));

    // .rsk file paths
    let SETTINGS_RSK: PathBuf = RSK_DIR.join(PathBuf::from("settings.rsk"));

    // cwd file paths
    let TUTORIAL_RSK: PathBuf = CWD.join(PathBuf::from("tutorial.rsk"));
    let TUTORIAL_MD: PathBuf = CWD.join(PathBuf::from("tutorial.md"));
    let CAPITOLS_CSV: PathBuf = CWD.join(PathBuf::from("capitols.csv"));
    let EXERCISE_HTM: PathBuf = CWD.join(PathBuf::from("exercise.htm"));

    // docs file paths
    let PURPOSE_RSK: PathBuf = DOCS_DIR.join(PathBuf::from("purpose.rsk"));
    let HIGH_LEVEL_RSK: PathBuf = DOCS_DIR.join(PathBuf::from("high_level.rsk"));
    let LOAD_RSK: PathBuf = DOCS_DIR.join(PathBuf::from("load.rsk"));

    // src file paths
    let INIT_PY: PathBuf = SRC_DIR.join(PathBuf::from("__init__.py"));
    let LOAD_PY: PathBuf = SRC_DIR.join(PathBuf::from("load.py"));

    // tests file paths
    let TEST_INIT_PY: PathBuf = TESTS_DIR.join(PathBuf::from("__init__.py"));
    let TEST_LOAD_PY: PathBuf = TESTS_DIR.join(PathBuf::from("test_load.py"));

    let PART_1_DIRS: HashSet<PathBuf> = HashSet::from_iter(vec![
        RSK_DIR.clone()]);
    let mut PART_2_DIRS: HashSet<PathBuf> = HashSet::from_iter(vec![
        RSK_DIR.clone(), DOCS_DIR.clone()]);
    let mut PART_3_DIRS: HashSet<PathBuf> = HashSet::from_iter(vec![
        RSK_DIR.clone(), DOCS_DIR.clone(), SRC_DIR.clone(), TESTS_DIR.clone()]);

    let PART_1_FILES: HashSet<PathBuf> = HashSet::from_iter(vec![
        SETTINGS_RSK.clone(), TUTORIAL_RSK.clone()]);

    let PART_2_FILES: HashSet<PathBuf> = HashSet::from_iter(vec![
        SETTINGS_RSK.clone(), TUTORIAL_MD.clone(), CAPITOLS_CSV.clone(),
        EXERCISE_HTM.clone(), PURPOSE_RSK.clone(), HIGH_LEVEL_RSK.clone()]);

    let mut PART_3_FILES: HashSet<PathBuf> = HashSet::from_iter(vec![
        LOAD_RSK.clone(), INIT_PY.clone(), LOAD_PY.clone(),
        TEST_INIT_PY.clone(), TEST_LOAD_PY.clone()]);

    PART_3_FILES.extend(PART_2_FILES.iter().cloned());

    let mut ALL_FILES: HashSet<PathBuf> = HashSet::new();
    ALL_FILES.extend(PART_1_FILES.iter().cloned());
    ALL_FILES.extend(PART_3_FILES.iter().cloned());

    let already_tutorial = match fs::File::open(SETTINGS_RSK) {
        Ok(mut f) => {
            let mut buffer = [0; 14];
            match f.read_exact(&mut buffer) {
                Ok(_) => buffer == "#TUTORIAL=true".as_ref(),
                Err(_) => false,
            }
        }
        Err(_) => false,
    };
    if !already_tutorial {
        debug!("cwd is not a tutorial");
        // make sure the directory is empty -- we don't want to
        // delete anything we shouldn't
        match try!(fs::read_dir(CWD)).next() {
            Some(_) => {
                println!("ERROR: can only start the rsk tutorial in an empty directory");
                return Ok(());
            },
            None => {},  // empty directory, what we want
        }
    } else {
        debug!("cwd is already a tutorial")
    }

    Ok(())
}
