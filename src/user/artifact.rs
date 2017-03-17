//! raw text loading

use toml::{Value, Table, Decoder};
use rustc_serialize::Decodable;

use dev_prefix::*;
use types::*;
use user::types::*;
use user::save::ProjectText;
use utils::parse_toml;

// Public Methods

// TODO: making this parallel should be easy and dramatically improve performance:
// - recursing through the directory, finding all the paths to files
//     (and adding dirs to loaded_dirs)
// - loading the files in parallel (IO bound)
// - resolving all settings at the end
/// recursively load the directory into text files, making sure
/// not to load files that have already been loaded
pub fn load_text(ptext: &mut ProjectText,
                 load_path: &Path,
                 loaded_paths: &mut HashSet<PathBuf>)
                 -> Result<()> {
    let mut files_to_load: Vec<PathBuf> = Vec::new();
    let mut dirs_to_load: Vec<PathBuf> = Vec::new();
    let ptype = load_path.metadata()
        .chain_err(|| format!("cannot get type: {}", load_path.display()))?
        .file_type();
    if ptype.is_dir() {
        // just read text from all .toml files in the directory
        // and record which directories need to be loaded
        // TODO: replace with walk_dir
        let dir_entries = fs::read_dir(load_path)
            .chain_err(|| format!("could not get dir: {}", load_path.display()))?;
        for entry in dir_entries.filter_map(|e| e.ok()) {
            let fpath = entry.path();
            if loaded_paths.contains(&fpath) {
                continue;
            }
            loaded_paths.insert(fpath.to_path_buf());
            let ftype = entry.file_type()
                .chain_err(|| format!("error reading type: {}", fpath.display()))?;
            if ftype.is_dir() {
                dirs_to_load.push(fpath.clone());
            } else if ftype.is_file() {
                files_to_load.push(fpath.clone());
            }
        }
    } else if ptype.is_file() {
        files_to_load.push(load_path.to_path_buf());
    } else {
        let msg = format!("invalid path: {}", load_path.display());
        return Err(ErrorKind::PathNotFound(msg).into());
    }

    for fpath in files_to_load {
        let ext = match fpath.extension() {
            None => continue,
            Some(ext) => ext,
        };
        if ext != "toml" {
            // only load toml files
            continue;
        }
        let mut text = String::new();
        let mut fp =
            fs::File::open(&fpath).chain_err(|| format!("error opening: {}", fpath.display()))?;
        fp.read_to_string(&mut text)
            .chain_err(|| format!("Error loading path {}", fpath.display()))?;
        ptext.files.insert(fpath.to_path_buf(), text);
    }
    for dir in dirs_to_load {
        load_text(ptext, dir.as_path(), loaded_paths)?;
    }
    Ok(())
}


/// method to convert `ProjectText` -> `Project`
/// Project may be extended by more than one `ProjectText`
pub fn extend_text(project: &mut Project, project_text: &ProjectText) -> Result<u64> {
    let mut count = 0;
    for (path, text) in &project_text.files {
        count += load_toml(path, text, project)?;
    }
    Ok(count)
}


// Public For Tests

/// Given text load the artifacts
pub fn load_toml(path: &Path, text: &str, project: &mut Project) -> Result<u64> {
    // parse the text
    let table = parse_toml(text)?;
    let mut num_loaded: u64 = 0;
    project.files.insert(path.to_path_buf());

    for (name, value) in &table {
        let aname = Name::from_str(name)?;
        // get the artifact table
        let art_tbl: &Table = match *value {
            Value::Table(ref t) => t,
            _ => {
                let msg = format!("All top-level values must be a table: {}", name);
                return Err(ErrorKind::Load(msg).into());
            }
        };
        // check for overlap
        if let Some(overlap) = project.artifacts.get(&aname) {
            let msg = format!("Overlapping key found <{}> other key at: {}",
                              name,
                              overlap.path.display());
            return Err(ErrorKind::Load(msg).into());
        }
        let artifact = from_table(&aname, path, art_tbl)?;
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
        let table = try!(parse_toml(toml));
        if table.len() != 1 {
            return Err(ErrorKind::Load("must contain a single table".to_string()).into());
        }
        let (name, value) = table.iter().next().unwrap();
        let name = try!(Name::from_str(name));
        let value = match *value {
            Value::Table(ref t) => t,
            _ => return Err(ErrorKind::Load("must contain a single table".to_string()).into()),
        };
        let artifact = try!(from_table(&name, &Path::new("from_str"), value));
        Ok((Arc::new(name), artifact))
    }
}

// Private

/// Create an artifact object from a toml Table
fn from_table(name: &Name, path: &Path, tbl: &Table) -> Result<Artifact> {
    let value = Value::Table(tbl.clone());
    let mut decoder = Decoder::new(value);
    let raw = match UserArtifact::decode(&mut decoder) {
        Ok(v) => v,
        Err(e) => return Err(ErrorKind::InvalidArtifact(name.to_string(), e.to_string()).into()),
    };
    if let Some(invalid) = decoder.toml {
        return Err(ErrorKind::InvalidArtifact(name.to_string(),
                                              format!("invalid attrs: {}", invalid))
                           .into());
    }
    let done = match raw.done {
        Some(s) => Done::Defined(s),
        None => Done::NotDone,
    };

    Ok(Artifact {
           path: path.to_path_buf(),
           text: raw.text.unwrap_or_default(),
           partof: Names::from_str(&raw.partof.unwrap_or_default())?,
           done: done,
           // calculated vars
           parts: HashSet::new(),
           completed: -1.0,
           tested: -1.0,
       })
}

#[cfg(test)]
mod tests {
    use toml::Parser;


    use super::*;
    use user::locs;
    use test_data;

    #[test]
    /// this is just a sanity-check/playground for how to implement
    /// loading using toml decoder
    fn test_load_raw_impl() {
        let text = r#"
        [REQ-one]
        partof = "REQ-1"
        text = '''
        I am text
        '''
        "#;
        let file_table = Parser::new(text).parse().unwrap();
        let mut artifacts: HashMap<String, UserArtifact> = HashMap::new();
        for (name, value) in file_table.iter() {
            let mut decoder = Decoder::new(value.clone());
            let raw = UserArtifact::decode(&mut decoder).unwrap();
            artifacts.insert(name.clone(), raw);
        }
        assert_eq!(artifacts.get("REQ-one").unwrap().text,
                   Some("        I am text\n        ".to_string()));
        assert_eq!(artifacts.get("REQ-one").unwrap().partof,
                   Some("REQ-1".to_string()));
    }

    #[test]
    fn test_load_toml() {
        let mut p = Project::default();

        let path = PathBuf::from("hi/there");

        // #TST-load-invalid
        assert!(load_toml(&path, test_data::TOML_BAD, &mut p).is_err());
        assert!(load_toml(&path, test_data::TOML_BAD_JSON, &mut p).is_err());
        assert!(load_toml(&path, test_data::TOML_BAD_ATTR1, &mut p).is_err());
        assert!(load_toml(&path, test_data::TOML_BAD_ATTR2, &mut p).is_err());
        assert!(load_toml(&path, test_data::TOML_BAD_NAMES1, &mut p).is_err());
        assert!(load_toml(&path, test_data::TOML_BAD_NAMES2, &mut p).is_err());

        // basic loading unit tests
        let num = load_toml(&path, test_data::TOML_RST, &mut p).unwrap();

        let locs = HashMap::from_iter(vec![(Name::from_str("SPC-foo").unwrap(), Loc::fake())]);
        let dne_locs = locs::attach_locs(&mut p.artifacts, locs).unwrap();
        assert_eq!(num, 8);
        assert_eq!(dne_locs.len(), 0);
        assert!(p.artifacts.contains_key(&Name::from_str("REQ-foo").unwrap()));
        assert!(p.artifacts.contains_key(&Name::from_str("SPC-foo").unwrap()));
        assert!(p.artifacts.contains_key(&Name::from_str("RSK-foo").unwrap()));
        assert!(p.artifacts.contains_key(&Name::from_str("TST-foo").unwrap()));
        assert!(p.artifacts.contains_key(&Name::from_str("SPC-bar").unwrap()));

        // will be loaded later
        assert!(!p.artifacts.contains_key(&Name::from_str("REQ-baz").unwrap()));
        assert!(!p.artifacts.contains_key(&Name::from_str("RSK-foo-2").unwrap()));
        assert!(!p.artifacts.contains_key(&Name::from_str("TST-foo-2").unwrap()));

        {
            // test to make sure default attrs are correct
            let rsk_foo = Name::from_str("RSK-foo").unwrap();
            let art = p.artifacts.get(&rsk_foo).unwrap();
            assert_eq!(rsk_foo.ty, Type::RSK);
            assert_eq!(art.path, path);
            assert_eq!(art.text, "");
            let expected: Names = HashSet::new();
            assert_eq!(art.partof, expected);
            assert_eq!(art.done, Done::NotDone);
            assert_eq!(art.completed, -1.0);
            assert_eq!(art.tested, -1.0);

            // test non-defaults
            let spc_bar = Name::from_str("SPC-bar").unwrap();
            let art = p.artifacts.get(&spc_bar).unwrap();
            assert_eq!(spc_bar.ty, Type::SPC);
            assert_eq!(art.path, path);
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

            let spc_foo = Name::from_str("SPC-foo").unwrap();
            let art = p.artifacts.get(&spc_foo).unwrap();
            let expected = Done::Code(Loc::fake());
            assert_eq!(art.done, expected);
        }

        // must be loaded afterwards, uses already existing artifacts
        assert!(load_toml(&path, test_data::TOML_OVERLAP, &mut p).is_err());

        let num = load_toml(&path, test_data::TOML_RST2, &mut p).unwrap();
        assert_eq!(num, 3);
        assert!(p.artifacts.contains_key(&Name::from_str("REQ-baz").unwrap()));
        assert!(p.artifacts.contains_key(&Name::from_str("RSK-foo-2").unwrap()));
        assert!(p.artifacts.contains_key(&Name::from_str("TST-foo-2").unwrap()));
    }

}
