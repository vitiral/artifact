/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
use dev_prefix::*;
use types::*;
use cmd::types::*;


pub const SETTINGS_TOML: &'static [u8] = include_bytes!("data/settings-template.toml");

pub const PURPOSE_TOML: &'static [u8] = include_bytes!("data/purpose-template.toml");

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    // #SPC-cmd-init
    SubCommand::with_name("init")
        .about("Initialize an artifact repository in the current directory")
        .settings(&SUBCMD_SETTINGS)
}

pub fn run_cmd(path: &Path) -> Result<u8> {
    let mut read_dir = fs::read_dir(path).chain_err(|| format!("dir: {}", path.display()))?;
    let exists = read_dir.any(|e| match e {
        Err(_) => false,
        Ok(e) => if !e.file_type().unwrap().is_dir() {
            false
        } else {
            let p = e.path();
            let fname = p.file_name().unwrap().to_str().unwrap();
            fname == ".art"
        },
    });
    let repo = path.join(".art");
    let design = path.join("design");
    if !exists {
        fs::create_dir(&repo).chain_err(|| format!("Failed to create dir: {}", repo.display()))?;
        let _ = fs::create_dir(&design);

        // create settings
        let settings = repo.join("settings.toml");
        let purpose = design.join("purpose.toml");
        {
            fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&settings)
                .chain_err(|| format!("Failed to create file: {}", settings.display()))?
                .write_all(SETTINGS_TOML.as_ref())
                .unwrap();
            println!("Artifact project initialized at {}", settings.display());
            if let Ok(mut f) = fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&purpose)
            {
                f.write_all(PURPOSE_TOML.as_ref()).unwrap();
                println!(
                    "Artifact created initial design.toml at {}",
                    design.display()
                )
            }
        }
    } else {
        println!("Artifact already initialized at {}", repo.display());
    }
    Ok(0)
}
