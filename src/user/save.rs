//! loadrs
//! loading of raw artifacts from files and text

use toml;
use difference::Changeset;
use std::collections::BTreeMap;

use dev_prefix::*;
use types::*;
use user::types::*;
use user::markdown;

// Public Struct

/// struct for representing a project as just a collection of
/// Path and String values, used for loading/formatting/saving files
#[derive(Debug, PartialEq)]
pub struct ProjectText {
    pub origin: PathBuf,
    pub files: HashMap<PathBuf, String>,
}

/// used for finding the difference between files in a project
pub enum PathDiff {
    DoesNotExist,
    NotUtf8,
    Changeset(Changeset),
    None,
}

impl Default for ProjectText {
    fn default() -> ProjectText {
        ProjectText {
            origin: PathBuf::from("INVALID-ORIGIN"),
            files: HashMap::default(),
        }
    }
}

impl ProjectText {
    /// convert a `Project` -> `ProjectText`
    pub fn from_project(project: &Project) -> Result<ProjectText> {
        let mut files: HashMap<PathBuf, BTreeMap<String, UserArtifact>> = HashMap::new();

        // we just go through each item, growing `files` as necessary
        // TODO: how to make the equivalent of a yielding function,
        // to not copy/paste the path filtering code.
        for (name, artifact) in &project.artifacts {
            // insert artifact into a table
            if !files.contains_key(&artifact.def) {
                files.insert(artifact.def.clone(), BTreeMap::new());
            }
            let tbl = files.get_mut(&artifact.def).unwrap();

            let partof = {
                let mut auto_partof = name.named_partofs();
                if !name.is_root() {
                    auto_partof.push(name.parent().expect("no parent"));
                }
                let auto_partof: HashSet<Name> = HashSet::from_iter(auto_partof.drain(0..));
                let mut strs = artifact
                    .partof
                    .iter()
                    .filter(|p| !auto_partof.contains(p))
                    .map(|p| p.raw.clone())
                    .collect::<Vec<_>>();
                if strs.is_empty() {
                    None
                } else if strs.len() == 1 {
                    let s = strs.drain(0..).next().unwrap();
                    Some(UserPartof::Single(s))
                } else {
                    strs.sort();
                    Some(UserPartof::Multi(strs))
                }
            };

            let raw = UserArtifact {
                partof: partof,
                text: if artifact.text.is_empty() {
                    None
                } else {
                    Some(artifact.text.clone())
                },
                done: if let Done::Defined(ref d) = artifact.done {
                    Some(d.clone())
                } else {
                    None
                },
            };
            tbl.insert(name.raw.clone(), raw);
        }

        // convert Values to text
        let mut text: HashMap<PathBuf, String> = HashMap::new();
        for (p, v) in files.drain() {
            let s = match p.extension()
                .expect(&format!("no extension: {:?}", p))
                .to_str()
                .expect("extension not utf8")
            {
                "toml" => {
                    let mut s = String::new();
                    v.serialize(&mut toml::Serializer::pretty(&mut s))
                        .expect("serialize");
                    s
                }
                "md" => markdown::to_markdown(&v)?,
                ext => panic!("internal error: unkown extension {}", ext),
            };
            text.insert(p, s);
        }

        // files that exist but have no data need to also be
        // written
        for p in &project.files {
            if !text.contains_key(p) {
                text.insert(p.clone(), String::with_capacity(0));
            }
        }

        Ok(ProjectText {
            files: text,
            origin: project.origin.clone(),
        })
    }

    /// dump text to origin
    pub fn dump(&self) -> Result<()> {
        for (path, text) in &self.files {
            debug!("Writing to {}", path.display());
            // create the directory
            if let Err(err) = fs::create_dir_all(path.parent().expect("path not file")) {
                match err.kind() {
                    io::ErrorKind::AlreadyExists => {}
                    _ => return Err(err.into()),
                }
            }
            let mut f = fs::File::create(path)?;
            f.write_all(text.as_bytes())?;
        }
        Ok(())
    }

    /// get a hash table with the diff values of the files
    /// in a project to what currently exists
    pub fn diff(&self) -> Result<HashMap<PathBuf, PathDiff>> {
        let mut out: HashMap<PathBuf, PathDiff> = HashMap::new();
        for (path, text) in &self.files {
            debug!("Diffing: {}", path.display());
            let mut f = match fs::File::open(path) {
                Ok(f) => f,
                Err(_) => {
                    out.insert(path.clone(), PathDiff::DoesNotExist);
                    continue;
                }
            };

            let mut bytes = Vec::new();
            f.read_to_end(&mut bytes)?;

            // get the original text
            let original = match str::from_utf8(&bytes) {
                Ok(s) => s,
                Err(_) => {
                    out.insert(path.clone(), PathDiff::NotUtf8);
                    continue;
                }
            };

            let ch = Changeset::new(original, text, "\n");
            let d = if ch.distance == 0 {
                PathDiff::None
            } else {
                PathDiff::Changeset(ch)
            };
            out.insert(path.clone(), d);
        }
        Ok(out)
    }
}
