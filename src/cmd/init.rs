/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2017  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */
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
                                  Ok(e) => {
                                      if !e.file_type().unwrap().is_dir() {
                                          false
                                      } else {
                                          let p = e.path();
                                          let fname = p.file_name()
            .unwrap()
            .to_str()
            .unwrap();
                                          fname == ".art"
                                      }
                                  }
                              });
    let repo = path.join(".art");
    let design = path.join("design");
    if !exists {
        fs::create_dir(&repo).chain_err(|| format!("create dir: {}", repo.display()))?;
        let _ = fs::create_dir(&design);

        // create settings
        let settings = repo.join("settings.toml");
        let purpose = design.join("purpose.toml");
        {
            fs::OpenOptions::new()
                .create_new(true)
                .write(true)
                .open(&settings)
                .chain_err(|| format!("create file: {}", settings.display()))?
                .write_all(SETTINGS_TOML.as_ref())
                .unwrap();
            println!("art initialized at {}", settings.display());
            if let Ok(mut f) = fs::OpenOptions::new().create_new(true).write(true).open(&purpose) {
                f.write_all(PURPOSE_TOML.as_ref()).unwrap();
                println!("art created initial design.toml at {}", design.display())
            }
        }
    } else {
        println!("artifact already initialized at {}", repo.display());
    }
    Ok(0)
}
