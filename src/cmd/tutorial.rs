#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

pub use std::path::{PathBuf, Path};
pub use std::fs;
pub use std::io::Read;
pub use std::env;
use super::types::*;

use super::data;

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
pub fn get_tutorial_cmd(matches: &ArgMatches) -> Result<u8, String> {
    let part = match matches.value_of("part").unwrap_or("1").parse::<u8>() {
        Ok(p) => p,
        Err(e) => return Err(e.to_string()),
    };
    if 1 <= part && part <= 3 {
        Ok(part)
    } else {
        let mut msg = String::new();
        write!(msg, "part must be 1, 2 or 3. Got: {}", part).unwrap();
        Err(msg)
    }
}

fn remove_files_force(files: &HashSet<PathBuf>) {
    for p in files.iter() {
        trace!("removing file: {}", p.display());
        let _ = fs::remove_file(p);
    }
}

fn remove_dirs_force(files: &HashSet<PathBuf>) {
    for p in files.iter() {
        trace!("removing dir: {}", p.display());
        let _ = fs::remove_dir_all(p);
    }
}

fn write_file(path: &PathBuf, data: &str) -> io::Result<()> {
    trace!("creating file: {}", path.display());
    let mut f = try!(fs::File::create(path.as_path()));
    try!(f.write_all(data.as_ref()));
    Ok(())
}

pub fn do_tutorial(part: u8) -> io::Result<()> {
    let CWD: PathBuf = env::current_dir().unwrap();
    let RSK_DIR: PathBuf = CWD.join(PathBuf::from(".rsk"));
    let DOCS_DIR: PathBuf = CWD.join(PathBuf::from("docs"));
    let SRC_DIR: PathBuf = CWD.join(PathBuf::from("flash"));
    let TESTS_DIR: PathBuf = SRC_DIR.join(PathBuf::from("tests"));

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
    let EXAMPLE_CSV: PathBuf = TESTS_DIR.join(PathBuf::from("example.csv"));

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

    let already_tutorial = match fs::File::open(&SETTINGS_RSK) {
        Ok(mut f) => {
            let mut buffer = [0; 14];
            match f.read_exact(&mut buffer) {
                Ok(_) => buffer == "#TUTORIAL=true".as_ref(),
                Err(_) => false,
            }
        }
        Err(_) => false,
    };
    debug!("running with cwd: {}", CWD.display());
    if !already_tutorial {
        debug!("cwd is not a tutorial");
        // make sure the directory is empty -- we don't want to
        // delete anything we shouldn't
        match try!(fs::read_dir(&CWD)).next() {
            Some(_) => {
                println!("ERROR: can only start the rsk tutorial in an empty directory");
                return Ok(());
            },
            None => {},  // empty directory, what we want
        }
    } else {
        debug!("cwd is already a tutorial")
    }
    debug!("running tutorial at part: {}", part);

    remove_files_force(&ALL_FILES);
    remove_dirs_force(&PART_3_DIRS);
    if part == 1 {
        try!(fs::create_dir(&RSK_DIR));
        try!(write_file(&SETTINGS_RSK, data::settings_1::data));
        try!(write_file(&TUTORIAL_RSK, data::tutorial_rsk::data));
        println!("Tutorial part 1: open tutorial.rsk with a text editor");
    } else if part == 2 || part == 3 {
        try!(fs::create_dir(&RSK_DIR));
        try!(fs::create_dir(&DOCS_DIR));
        try!(write_file(&TUTORIAL_MD, data::tutorial_md::data));
        try!(write_file(&CAPITOLS_CSV, data::capitols_csv::data));
        try!(write_file(&EXERCISE_HTM, data::exercise_htm::data));
        try!(write_file(&PURPOSE_RSK, data::purpose_rsk::data));
        try!(write_file(&HIGH_LEVEL_RSK, data::high_level_rsk::data));
        if part == 2 {
            println!("Tutorial part 2: see tutorial.md part 2");
            try!(write_file(&SETTINGS_RSK, data::settings_2::data));
        } else if part == 3 {
            println!("Tutorial part 3: see tutorial.md part 3");
            try!(fs::create_dir(&SRC_DIR));
            try!(fs::create_dir(&TESTS_DIR));
            try!(write_file(&SETTINGS_RSK, data::settings_3::data));
            try!(write_file(&LOAD_RSK, data::load_rsk::data));
            try!(write_file(&LOAD_PY, data::load_py::data));
            try!(write_file(&TEST_LOAD_PY, data::test_load_py::data));
            try!(write_file(&EXAMPLE_CSV, data::example_csv::data));
            try!(write_file(&INIT_PY, ""));
            try!(write_file(&TEST_INIT_PY, ""));
        }
    }
    Ok(())
}
