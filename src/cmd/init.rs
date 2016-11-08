/*  rst: the requirements tracking tool made for developers
    Copyright (C) 2016  Garrett Berg <@vitiral, vitiral@gmail.com>

    This program is free software: you can redistribute it and/or modify
    it under the terms of the Lesser GNU General Public License as published 
    by the Free Software Foundation, either version 3 of the License, or
    (at your option) any later version.

    This program is distributed in the hope that it will be useful,
    but WITHOUT ANY WARRANTY; without even the implied warranty of
    MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
    GNU General Public License for more details.

    You should have received a copy of the Lesser GNU General Public License
    along with this program.  If not, see <http://www.gnu.org/licenses/>.
*/
pub use std::fs;
use super::types::*;

const SETTINGS_TOML: &'static str = r#"[settings]
artifact_paths = ["{repo}/reqs"]
code_paths = []
exclude_code_paths = []

[globals]
# This is where you define variables
# There are two variables that can be used anywhere:
# - {cwd}: the path to the directory of the file using it
# - {repo}: the path to the current repository, which is the closest
#    directory (searching down) that contains a ".rst" folder
"#;

const PURPOSE_TOML: &'static str = r#"# project purpose and definition documentation
[REQ-purpose]
text = '''
The purpose of this project is...
'''
"#;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    // #SPC-init
    SubCommand::with_name("init")
        .about("initiailze the repository in the cwd")
        .settings(&[AS::DeriveDisplayOrder, AS::ColoredHelp])
}

pub fn do_init(path: &Path) -> io::Result<()> {
    let mut read_dir = try!(fs::read_dir(path));
    let exists = read_dir.any(|e|
        match e {
            Err(_) => false,
            Ok(e) => {
                if !e.file_type().unwrap().is_dir() {
                    false
                } else {
                    let p = e.path();
                    let fname = p.file_name().unwrap().to_str().unwrap();
                    fname == ".rst"
                }
            }
        });
    let repo = path.join(".rst");
    let reqs = path.join("reqs");
    if !exists {
        try!(fs::create_dir(&repo));
        let _ = fs::create_dir(&reqs);

        // create settings
        let settings = repo.join("settings.toml");
        let purpose = reqs.join("purpose.toml");
        let mut f = try!(fs::File::create(&settings));
        f.write_all(SETTINGS_TOML.as_ref()).unwrap();
        let mut f = try!(fs::File::create(purpose));
        f.write_all(PURPOSE_TOML.as_ref()).unwrap();
        println!("rst initialized at {} with artifacts at {}",
                 settings.display(), reqs.display());
    } else {
        println!("rst already initialized at {}", repo.display());
    }
    Ok(())
}
