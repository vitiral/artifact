/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
#![allow(non_upper_case_globals)]
#![allow(non_snake_case)]

use dev_prefix::*;
use types::*;
use super::types::*;

/// return the cmdline subcommand
pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("tutorial")
        .about("Start the interactive tutorial")
        .settings(&SUBCMD_SETTINGS)
        .arg(
            Arg::with_name("part").help("The section of the tutorial to view"),
        )
}

/// parse the matches to create the cmd
pub fn get_cmd(matches: &ArgMatches) -> result::Result<u8, String> {
    let part = match matches.value_of("part").unwrap_or("1").parse::<u8>() {
        Ok(p) => p,
        Err(e) => return Err(e.to_string()),
    };
    if 1 <= part && part <= 5 {
        Ok(part)
    } else {
        let mut msg = String::new();
        write!(msg, "part must be between 1 and 5. Got: {}", part).unwrap();
        Err(msg)
    }
}

/// remove files with logging, ignore errors
fn remove_files_force(files: &HashSet<PathBuf>) {
    for p in files.iter() {
        trace!("Removing file: {}", p.display());
        let _ = fs::remove_file(p);
    }
}

/// write to a file with logging
fn write_file(path: &PathBuf, data: &[u8]) -> io::Result<()> {
    trace!("Creating file: {}", path.display());
    let mut f = try!(fs::File::create(path.as_path()));
    try!(f.write_all(data));
    Ok(())
}

/// create a directory with logging, ignore any errors
fn create_dir(path: &PathBuf) {
    trace!("Creating dir: {}", path.display());
    let _ = fs::create_dir(path);
}

// static files that are compiled into the binary for command `tutorial`
pub const D_TUTORIAL_TOML: &'static [u8] = include_bytes!("data/tutorial.toml");
pub const D_TUTORIAL_MD: &'static [u8] = include_bytes!("data/tutorial.md");

pub const D_CAPITOLS_CSV: &'static [u8] = include_bytes!("data/capitols.csv");
pub const D_FLASH_CARD_CHALLENGE_HTM: &'static [u8] = include_bytes!(
    "data/flash-card-challenge.\
     htm"
);
pub const D_PURPOSE_TOML: &'static [u8] = include_bytes!("data/purpose.toml");

pub const D_LOAD_1_PY: &'static [u8] = include_bytes!("data/load-1.py");
pub const D_LOAD_1_TOML: &'static [u8] = include_bytes!("data/load-1.toml");
pub const D_LOAD_2_PY: &'static [u8] = include_bytes!("data/load-2.py");
pub const D_LOAD_2_TOML: &'static [u8] = include_bytes!("data/load-2.toml");
pub const D_TEST_LOAD_PY: &'static [u8] = include_bytes!("data/test_load.py");
pub const D_TEST_DATA_CSV: &'static [u8] = include_bytes!("data/test_data.csv");

pub const D_SETTINGS_1_TOML: &'static [u8] = include_bytes!("data/settings-1.toml");
pub const D_SETTINGS_2_TOML: &'static [u8] = include_bytes!("data/settings-2.toml");
pub const D_SETTINGS_4_TOML: &'static [u8] = include_bytes!("data/settings-4.toml");



/// run the tutorial
/// partof: #SPC-cmd-tutorial
pub fn run_cmd(cwd: &Path, part: u8) -> Result<u8> {
    // let CWD: PathBuf = env::current_dir().unwrap();
    let RST_DIR: PathBuf = cwd.join(PathBuf::from(".art"));
    let DESIGN_DIR: PathBuf = cwd.join(PathBuf::from("design"));
    let SRC_DIR: PathBuf = cwd.join(PathBuf::from("flash"));
    let TESTS_DIR: PathBuf = SRC_DIR.join(PathBuf::from("tests"));

    // .toml file paths
    let SETTINGS_TOML: PathBuf = RST_DIR.join(PathBuf::from("settings.toml"));

    // cwd file paths
    let TUTORIAL_TOML: PathBuf = cwd.join(PathBuf::from("tutorial.toml"));
    let TUTORIAL_MD: PathBuf = cwd.join(PathBuf::from("tutorial.md"));
    let CAPITOLS_CSV: PathBuf = cwd.join(PathBuf::from("capitols.csv"));
    let FLASH_CARD_CHALLENGE_HTM: PathBuf = cwd.join(PathBuf::from("flash_card_challenge.htm"));

    // design file paths
    let PURPOSE_TOML: PathBuf = DESIGN_DIR.join(PathBuf::from("purpose.toml"));
    let LOAD_TOML: PathBuf = DESIGN_DIR.join(PathBuf::from("load.toml"));

    // src file paths
    let INIT_PY: PathBuf = SRC_DIR.join(PathBuf::from("__init__.py"));
    let LOAD_PY: PathBuf = SRC_DIR.join(PathBuf::from("load.py"));

    // tests file paths
    let TEST_INIT_PY: PathBuf = TESTS_DIR.join(PathBuf::from("__init__.py"));
    let TEST_LOAD_PY: PathBuf = TESTS_DIR.join(PathBuf::from("test_load.py"));
    let TEST_DATA_CSV: PathBuf = TESTS_DIR.join(PathBuf::from("example.csv"));

    let PART_1_FILES: HashSet<PathBuf> = HashSet::from_iter(vec![SETTINGS_TOML.clone()]);

    let PART_2_FILES: HashSet<PathBuf> = HashSet::from_iter(vec![
        SETTINGS_TOML.clone(),
        TUTORIAL_MD.clone(),
        CAPITOLS_CSV.clone(),
        FLASH_CARD_CHALLENGE_HTM.clone(),
        PURPOSE_TOML.clone(),
    ]);

    let mut PART_3_FILES: HashSet<PathBuf> = HashSet::from_iter(vec![
        LOAD_TOML.clone(),
        INIT_PY.clone(),
        LOAD_PY.clone(),
        TEST_INIT_PY.clone(),
        TEST_LOAD_PY.clone(),
    ]);

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
    debug!("Running with cwd: {}", cwd.display());
    if !already_tutorial {
        debug!("Cwd is not a tutorial");
        // make sure the directory is empty -- we don't want to
        // delete anything we shouldn't
        if fs::read_dir(&cwd)
            .chain_err(|| format!("Could not read dir: {}", cwd.display()))?
            .next()
            .is_some()
        {
            println!(
                "ERROR: can only start the artifact tutorial in an empty directory. \
                 To make an empty directory and change-dir to it, run:\n    \
                 mkdir ~/tryrst; cd ~/tryrst"
            );
            return Ok(0);
        }
    } else {
        debug!("Cwd is already a tutorial")
    }
    debug!("Running tutorial at part: {}", part);

    remove_files_force(&ALL_FILES);
    create_dir(&RST_DIR);
    println!("## Tutorial Loaded!");
    if !TUTORIAL_TOML.exists() {
        try!(write_file(&TUTORIAL_TOML, D_TUTORIAL_TOML));
    }
    if !TUTORIAL_MD.exists() {
        try!(write_file(&TUTORIAL_MD, D_TUTORIAL_MD));
    }
    if part == 1 {
        // stage 1: example toml file with self-description
        println!("  Tutorial part 1: open tutorial.toml with a text editor");
        try!(write_file(&SETTINGS_TOML, D_SETTINGS_1_TOML));
    } else {
        create_dir(&DESIGN_DIR);
        try!(write_file(&CAPITOLS_CSV, D_CAPITOLS_CSV));
        try!(write_file(
            &FLASH_CARD_CHALLENGE_HTM,
            D_FLASH_CARD_CHALLENGE_HTM,
        ));
        try!(write_file(&PURPOSE_TOML, D_PURPOSE_TOML));
        if part == 2 {
            // stage 2: purpose document
            println!("  Tutorial part 2: open tutorial.md with a text editor and see part 2");
            try!(write_file(&SETTINGS_TOML, D_SETTINGS_2_TOML));
        } else {
            if part == 3 {
                // stage 3: load.toml detailed design
                println!("  Tutorial part 3: open tutorial.md with a text editor and see part 3");
            }
            try!(write_file(&SETTINGS_TOML, D_SETTINGS_2_TOML)); // same settings as 2
            if part != 5 {
                try!(write_file(&LOAD_TOML, D_LOAD_1_TOML));
            }
            if part >= 4 {
                // stage 4: implementing and linking code
                create_dir(&SRC_DIR);
                create_dir(&TESTS_DIR);
                try!(write_file(&SETTINGS_TOML, D_SETTINGS_4_TOML));
                try!(write_file(&TEST_LOAD_PY, D_TEST_LOAD_PY));
                try!(write_file(&TEST_DATA_CSV, D_TEST_DATA_CSV));
                try!(write_file(&INIT_PY, b""));
                try!(write_file(&TEST_INIT_PY, b""));
                if part == 4 {
                    println!(
                        "  Tutorial part 4: open tutorial.md with a text editor and see \
                         part 4"
                    );
                    try!(write_file(&LOAD_PY, D_LOAD_1_PY));
                } else {
                    // stage 5: handling errors
                    println!(
                        "  Tutorial part 5: open tutorial.md with a text editor and see \
                         part 5"
                    );
                    try!(write_file(&LOAD_TOML, D_LOAD_2_TOML));
                    try!(write_file(&LOAD_PY, D_LOAD_2_PY));
                }
            }
        }
    }
    Ok(0)
}
