//! raw text loading

use toml;

use dev_prefix::*;
use types::*;
use user::types::*;
use user::save::ProjectText;
use user::markdown;
use utils::unique_id;

lazy_static!{
    // note: the `(?:\w+:)?` is an uncaptured group to allow
    // for special rendering in the form
    // [[dot:.subname]]
    pub static ref SUBNAME_RE: Regex = Regex::new(
        &format!(r"(?i)\[\[(?:\w+:)?\.([{}]+)\]\]", NAME_VALID_CHARS!())).unwrap();
}


// Public Methods

// TODO: making this parallel should be easy and dramatically improve performance:
// - recursing through the directory, finding all the paths to files
//     (and adding dirs to loaded_dirs)
// - loading the files in parallel (IO bound)
// - resolving all settings at the end
/// recursively load the directory into text files, making sure
/// not to load files that have already been loaded
pub fn load_file_path(
    ptext: &mut ProjectText,
    load_path: &Path,
    loaded_paths: &mut HashSet<PathBuf>,
) -> Result<()> {
    let mut files_to_load: Vec<PathBuf> = Vec::new();
    let mut dirs_to_load: Vec<PathBuf> = Vec::new();
    let ptype = load_path
        .metadata()
        .chain_err(|| format!("Cannot get type: {}", load_path.display()))?
        .file_type();
    if ptype.is_dir() {
        // just read text from all .toml files in the directory
        // and record which directories need to be loaded
        // TODO: replace with walk_dir
        let dir_entries = fs::read_dir(load_path)
            .chain_err(|| format!("Could not get dir: {}", load_path.display()))?;
        for entry in dir_entries.filter_map(|e| e.ok()) {
            let fpath = entry.path();
            if loaded_paths.contains(&fpath) {
                continue;
            }
            loaded_paths.insert(fpath.to_path_buf());
            let ftype = entry
                .file_type()
                .chain_err(|| format!("Error reading type: {}", fpath.display()))?;
            if ftype.is_dir() {
                dirs_to_load.push(fpath.clone());
            } else if ftype.is_file() {
                files_to_load.push(fpath.clone());
            }
        }
    } else if ptype.is_file() {
        files_to_load.push(load_path.to_path_buf());
    } else {
        let msg = format!("Invalid path: {}", load_path.display());
        return Err(ErrorKind::PathNotFound(msg).into());
    }

    for fpath in files_to_load {
        let ext = match fpath.extension() {
            None => continue,
            Some(ext) => ext,
        };
        if !(ext == "toml" || ext == "md") {
            // only load toml/md files
            continue;
        }
        let mut text = String::new();
        let mut fp =
            fs::File::open(&fpath).chain_err(|| format!("Error opening: {}", fpath.display()))?;
        fp.read_to_string(&mut text)
            .chain_err(|| format!("Error loading path {}", fpath.display()))?;
        ptext.files.insert(fpath.to_path_buf(), text);
    }
    for dir in dirs_to_load {
        load_file_path(ptext, dir.as_path(), loaded_paths)?;
    }
    Ok(())
}


fn parse_subnames(name: NameRc, text: &str) -> HashSet<SubName> {
    let mut out: HashSet<SubName> = HashSet::new();
    for cap in SUBNAME_RE.captures_iter(text) {
        // note: repeats are ignored (and are okay)
        out.insert(SubName::from_parts(
            name.clone(),
            cap.get(1).unwrap().as_str().into(),
        ));
    }
    out
}

/// method to convert `ProjectText` -> `Project`
/// Project may be extended by more than one `ProjectText`
pub fn extend_text(project: &mut Project, project_text: &ProjectText) -> Result<u64> {
    let mut count = 0;
    for (path, text) in &project_text.files {
        count += load_text(path, text, project)?;
    }
    Ok(count)
}


// Public For Tests

/// Given text load the artifacts
pub fn load_text(path: &Path, text: &str, project: &mut Project) -> Result<u64> {
    // parse the text
    let loaded: ::std::collections::BTreeMap<String, UserArtifact> = match path.extension()
        .expect(&format!("no extension: {:?}", path))
        .to_str()
        .expect("extension not utf8")
    {
        "toml" => toml::from_str(text)?,
        "md" => markdown::from_markdown(text)?,
        ext => panic!("internal error: unkown extension {}", ext),
    };
    let mut num_loaded: u64 = 0;
    project.files.insert(path.to_path_buf());

    for (name, user_artifact) in &loaded {
        let aname = Name::from_str(name)?;
        // check for overlap
        if let Some(overlap) = project.artifacts.get(&aname) {
            let msg = format!(
                "Overlapping key found <{}> other key at: {}",
                aname.raw,
                overlap.def.display()
            );
            return Err(ErrorKind::Load(msg).into());
        }
        let artifact = from_user_artifact(&aname, path, user_artifact.clone())?;
        project.artifacts.insert(Arc::new(aname), artifact);
        num_loaded += 1;
    }
    Ok(num_loaded)
}


#[cfg(test)]
impl Artifact {
    #[cfg(test)]
    #[allow(should_implement_trait)]
    /// from_str is mosty used to make testing and one-off development easier
    pub fn from_str(toml: &str) -> Result<(NameRc, Artifact)> {
        let mut loaded: HashMap<String, UserArtifact> = toml::from_str(toml)?;
        if loaded.len() != 1 {
            return Err(
                ErrorKind::Load("Must contain a single table".to_string()).into(),
            );
        }
        let (name, user_artifact) = loaded.drain().next().unwrap();
        let name = Name::from_string(name)?;
        let artifact = from_user_artifact(&name, &Path::new("from_str"), user_artifact)?;
        Ok((Arc::new(name), artifact))
    }
}

// Private

/// Create an artifact object from a toml Table
fn from_user_artifact(name: &Name, path: &Path, user_artifact: UserArtifact) -> Result<Artifact> {
    let done = match user_artifact.done {
        Some(s) => {
            if s == "" {
                return Err(
                    ErrorKind::InvalidAttr(
                        name.to_string(),
                        "`done` cannot be an empty string.".to_string(),
                    ).into(),
                );
            }
            Done::Defined(s)
        }
        None => Done::NotDone,
    };

    fn get_partof(raw: &str) -> Result<Names> {
        Names::from_str(raw)
    }

    let mut partof = if let Some(all_parts) = user_artifact.partof {
        match all_parts {
            UserPartof::Single(part) => get_partof(&part)?,
            UserPartof::Multi(parts) => {
                let mut out = HashSet::new();
                for part in parts {
                    let mut p = get_partof(&part)?;
                    out.extend(p.drain());
                }
                out
            }
        }
    } else {
        HashSet::new()
    };

    // Being a partof itself is a no-op
    partof.remove(name);

    let text = user_artifact.text.unwrap_or_default();
    let subnames = parse_subnames(Arc::new(name.clone()), &text);

    Ok(Artifact {
        id: unique_id(),
        revision: 0,
        def: path.to_path_buf(),
        text: text,
        subnames: subnames,
        partof: partof,
        done: done,
        // calculated vars
        parts: HashSet::new(),
        completed: -1.0,
        tested: -1.0,
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use user::locs;
    use test_data;

    #[test]
    fn test_load_toml() {
        let mut p = Project::default();

        let path = PathBuf::from("hi/there.toml");

        // #TST-project-invalid
        assert!(load_text(&path, test_data::TOML_BAD_JSON, &mut p).is_err());
        assert!(load_text(&path, test_data::TOML_BAD_ATTR1, &mut p).is_err());
        assert!(load_text(&path, test_data::TOML_BAD_ATTR2, &mut p).is_err());
        assert!(load_text(&path, test_data::TOML_BAD_NAMES1, &mut p).is_err());
        assert!(load_text(&path, test_data::TOML_BAD_NAMES2, &mut p).is_err());

        // Basic loading unit tests. Note NO processing is done
        // except attaching mocked locations
        let num = load_text(&path, test_data::TOML_RST, &mut p).unwrap();

        // FIXME: do something better for sublocs
        let locs = HashMap::from_iter(vec![(Name::from_str("SPC-foo").unwrap(), Loc::fake())]);
        let sublocs = HashMap::new();
        let (dne_locs, dne_sublocs) = locs::attach_locs(&mut p.artifacts, locs, sublocs).unwrap();
        assert_eq!(num, 9);
        assert_eq!(dne_locs.len(), 0);
        assert_eq!(dne_sublocs.len(), 0);
        assert!(
            p.artifacts
                .contains_key(&Name::from_str("REQ-foo").unwrap())
        );
        assert!(
            p.artifacts
                .contains_key(&Name::from_str("SPC-foo").unwrap())
        );
        assert!(
            p.artifacts
                .contains_key(&Name::from_str("TST-foo").unwrap())
        );
        assert!(
            p.artifacts
                .contains_key(&Name::from_str("SPC-bar").unwrap())
        );

        // will be loaded later
        assert!(!p.artifacts
            .contains_key(&Name::from_str("REQ-baz").unwrap()));
        assert!(!p.artifacts
            .contains_key(&Name::from_str("TST-foo-2").unwrap()));

        {
            // test to make sure default attrs are correct
            let spc_foo = Name::from_str("SPC-foo").unwrap();
            let art = p.artifacts.get(&spc_foo).unwrap();
            assert_eq!(spc_foo.ty, Type::SPC);
            assert_eq!(art.def, path);
            assert_eq!(art.text, "");
            assert_eq!(art.partof, HashSet::new());
            assert_eq!(art.done, Done::Code(FullLocs::fake()));
            assert_eq!(art.completed, -1.0);
            assert_eq!(art.tested, -1.0);
            assert_eq!(art.done, Done::Code(FullLocs::fake()));

            // test non-defaults
            let spc_bar = Name::from_str("SPC-bar").unwrap();
            let art = p.artifacts.get(&spc_bar).unwrap();
            assert_eq!(spc_bar.ty, Type::SPC);
            assert_eq!(art.def, path);
            assert_eq!(art.text, "bar");

            let expected = ["REQ-Foo", "REQ-Bar-1", "REQ-Bar-2"]
                .iter()
                .map(|n| NameRc::from_str(n).unwrap())
                .collect();
            assert_eq!(art.partof, expected);
            let expected = Done::Defined("bar is done".to_string());
            assert_eq!(art.done, expected);
            assert_eq!(art.completed, -1.0);
            assert_eq!(art.tested, -1.0);
        }

        // must be loaded afterwards, uses already existing artifacts
        assert!(load_text(&path, test_data::TOML_OVERLAP, &mut p).is_err());

        let num = load_text(&path, test_data::TOML_RST2, &mut p).unwrap();
        assert_eq!(num, 2);
        assert!(
            p.artifacts
                .contains_key(&Name::from_str("REQ-baz").unwrap())
        );
        assert!(
            p.artifacts
                .contains_key(&Name::from_str("TST-foo-2").unwrap())
        );
    }

    #[test]
    fn test_parse_subnames() {
        let name = Arc::new(Name::from_str("REQ-fake").unwrap());
        let text = r#"
        This is some text. Subname: [[.hello]]
        [[.goodbye]]
        "#;
        let subnames = parse_subnames(name.clone(), text);
        assert!(subnames.contains(&SubName::from_parts(name.clone(), "hello".into())));
        assert!(subnames.contains(&SubName::from_parts(name.clone(), "goodbye".into())));
    }

}
