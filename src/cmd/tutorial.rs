#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

pub use std::fs;
pub use std::io::Read;
pub use std::env;
use super::types::*;

use super::data;

/// return the cmdline subcommand
pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("tutorial")
        .about("start the interactive tutorial")
        .settings(&[AS::DeriveDisplayOrder, AS::ColoredHelp])
        .arg(Arg::with_name("part").help("the part to set the tutorial at"))
}

/// parse the matches to create the cmd
pub fn get_tutorial_cmd(matches: &ArgMatches) -> Result<u8, String> {
    let part = match matches.value_of("part").unwrap_or("1").parse::<u8>() {
        Ok(p) => p,
        Err(e) => return Err(e.to_string()),
    };
    if 1 <= part && part <= 4 {
        Ok(part)
    } else {
        let mut msg = String::new();
        write!(msg, "part must be between 1 and 4. Got: {}", part).unwrap();
        Err(msg)
    }
}

/// remove files with logging, ignore errors
fn remove_files_force(files: &HashSet<PathBuf>) {
    for p in files.iter() {
        trace!("removing file: {}", p.display());
        let _ = fs::remove_file(p);
    }
}

/// write to a file with logging
fn write_file(path: &PathBuf, data: &str) -> io::Result<()> {
    trace!("creating file: {}", path.display());
    let mut f = try!(fs::File::create(path.as_path()));
    try!(f.write_all(data.as_ref()));
    Ok(())
}

/// create a directory with logging, ignore any errors
fn create_dir(path: &PathBuf) {
    trace!("creating dir: {}", path.display());
    let _ = fs::create_dir(path);
}

/// run the tutorial
/// partof: #SPC-tutorial
pub fn do_tutorial(part: u8) -> io::Result<()> {
    let CWD: PathBuf = env::current_dir().unwrap();
    let RST_DIR: PathBuf = CWD.join(PathBuf::from(".rst"));
    let REQS_DIR: PathBuf = CWD.join(PathBuf::from("reqs"));
    let SRC_DIR: PathBuf = CWD.join(PathBuf::from("flash"));
    let TESTS_DIR: PathBuf = SRC_DIR.join(PathBuf::from("tests"));

    // .toml file paths
    let SETTINGS_TOML: PathBuf = RST_DIR.join(PathBuf::from("settings.toml"));

    // cwd file paths
    let TUTORIAL_TOML: PathBuf = CWD.join(PathBuf::from("tutorial.toml"));
    let TUTORIAL_MD: PathBuf = CWD.join(PathBuf::from("tutorial.md"));
    let CAPITOLS_CSV: PathBuf = CWD.join(PathBuf::from("capitols.csv"));
    let EXERCISE_HTM: PathBuf = CWD.join(PathBuf::from("flash_card_challenge.htm"));

    // reqs file paths
    let PURPOSE_TOML: PathBuf = REQS_DIR.join(PathBuf::from("purpose.toml"));
    let HIGH_LEVEL_TOML: PathBuf = REQS_DIR.join(PathBuf::from("high_level.toml"));
    let LOAD_TOML: PathBuf = REQS_DIR.join(PathBuf::from("load.toml"));

    // src file paths
    let INIT_PY: PathBuf = SRC_DIR.join(PathBuf::from("__init__.py"));
    let LOAD_PY: PathBuf = SRC_DIR.join(PathBuf::from("load.py"));

    // tests file paths
    let TEST_INIT_PY: PathBuf = TESTS_DIR.join(PathBuf::from("__init__.py"));
    let TEST_LOAD_PY: PathBuf = TESTS_DIR.join(PathBuf::from("test_load.py"));
    let EXAMPLE_CSV: PathBuf = TESTS_DIR.join(PathBuf::from("example.csv"));

    let PART_1_FILES: HashSet<PathBuf> = HashSet::from_iter(vec![SETTINGS_TOML.clone(),
                                                                 TUTORIAL_TOML.clone()]);

    let PART_2_FILES: HashSet<PathBuf> = HashSet::from_iter(vec![SETTINGS_TOML.clone(),
                                                                 TUTORIAL_MD.clone(),
                                                                 CAPITOLS_CSV.clone(),
                                                                 EXERCISE_HTM.clone(),
                                                                 PURPOSE_TOML.clone(),
                                                                 HIGH_LEVEL_TOML.clone()]);

    let mut PART_3_FILES: HashSet<PathBuf> = HashSet::from_iter(vec![LOAD_TOML.clone(),
                                                                     INIT_PY.clone(),
                                                                     LOAD_PY.clone(),
                                                                     TEST_INIT_PY.clone(),
                                                                     TEST_LOAD_PY.clone()]);

    PART_3_FILES.extend(PART_2_FILES.iter().cloned());

    let mut ALL_FILES: HashSet<PathBuf> = HashSet::new();
    ALL_FILES.extend(PART_1_FILES.iter().cloned());
    ALL_FILES.extend(PART_3_FILES.iter().cloned());

    let already_tutorial = match fs::File::open(&SETTINGS_TOML) {
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
                println!("ERROR: can only start the rst tutorial in an empty directory. \
                          To make an empty directory and change-dir to it, run:\n    \
                          mkdir ~/tryrst; cd ~/tryrst");
                return Ok(());
            }
            None => {}  // empty directory, what we want
        }
    } else {
        debug!("cwd is already a tutorial")
    }
    debug!("running tutorial at part: {}", part);

    remove_files_force(&ALL_FILES);
    create_dir(&RST_DIR);
    println!("## Tutorial Loaded!");
    if part == 1 {
        println!("  Tutorial part 1: open tutorial.toml with a text editor");
        try!(write_file(&SETTINGS_TOML, data::settings_1::DATA));
        try!(write_file(&TUTORIAL_TOML, data::tutorial_toml::DATA));
    } else {
        create_dir(&REQS_DIR);
        try!(write_file(&TUTORIAL_MD, data::tutorial_md::DATA));
        try!(write_file(&CAPITOLS_CSV, data::capitols_csv::DATA));
        try!(write_file(&EXERCISE_HTM, data::exercise_htm::DATA));
        try!(write_file(&PURPOSE_TOML, data::purpose_toml::DATA));
        try!(write_file(&HIGH_LEVEL_TOML, data::high_level_toml::DATA));
        if part == 2 {
            println!("  Tutorial part 2: open tutorial.md with a text editor and see part 2");
            try!(write_file(&SETTINGS_TOML, data::settings_2::DATA));
        } else {
            if part == 3 {
                println!("  Tutorial part 3: open tutorial.md with a text editor and see part 3");
            }
            try!(write_file(&SETTINGS_TOML, data::settings_2::DATA)); // same settings
            try!(write_file(&LOAD_TOML, data::load_toml::DATA));
            if part == 4 {
                println!("  Tutorial part 4: open tutorial.md with a text editor and see part 4");
                create_dir(&SRC_DIR);
                create_dir(&TESTS_DIR);
                try!(write_file(&SETTINGS_TOML, data::settings_4::DATA));
                try!(write_file(&LOAD_PY, data::load_py::DATA));
                try!(write_file(&TEST_LOAD_PY, data::test_load_py::DATA));
                try!(write_file(&EXAMPLE_CSV, data::example_csv::DATA));
                try!(write_file(&INIT_PY, ""));
                try!(write_file(&TEST_INIT_PY, ""));
            }
        }
    }
    Ok(())
}
