
pub use std::path::{PathBuf, Path};
pub use std::fs;
use super::types::*;

const SETTINGS_RSK: &'static str = r#"[settings]
artifact_paths = ["{repo}/reqs"]
code_paths = []
exclude_code_paths = []

[globals]
# This is where you define variables
# There are two variables that can be used anywhere:
# - {cwd}: the path to the directory of the file using it
# - {repo}: the path to the current repository, which is the closest
#    directory (searching down) that contains a ".rsk" folder
"#;

const PURPOSE_RSK: &'static str = r#"# project purpose and definition documentation
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
                    if fname == ".rsk" {
                        true
                    } else {
                        false
                    }
                }
            }
        });
    let repo = path.join(".rsk");
    let reqs = path.join("reqs");
    if !exists {
        try!(fs::create_dir(&repo));
        let _ = fs::create_dir(&reqs);

        // create settings
        let settings = repo.join("settings.rsk");
        let purpose = reqs.join("purpose.rsk");
        let mut f = try!(fs::File::create(&settings));
        f.write_all(SETTINGS_RSK.as_ref()).unwrap();
        let mut f = try!(fs::File::create(purpose));
        f.write_all(PURPOSE_RSK.as_ref()).unwrap();
        println!("rsk initialized at {} with artifacts at {}",
                 settings.display(), reqs.display());
    } else {
        println!("rsk already initialized at {}", repo.display());
    }
    Ok(())
}
