use dev_prefix::*;
use types::*;

lazy_static!{
    pub static ref ART_LOC: Regex = Regex::new(
        &format!(r"(?i)(?:#({}))|(\n)", NAME_VALID_STR)).unwrap();
}

// Public Methods

/// search through the `code_paths` in settings to find all valid locs
pub fn find_locs(settings: &Settings) -> Result<HashMap<Name, Loc>> {
    info!("parsing code files for artifacts...");
    let mut locs: HashMap<Name, Loc> = HashMap::new();
    let mut loaded: HashSet<PathBuf> =
        HashSet::from_iter(settings.exclude_code_paths.iter().map(|p| p.to_path_buf()));
    debug!("excluded code paths: {:?}", loaded);
    for path in &settings.code_paths {
        if loaded.contains(path) {
            continue;
        }
        debug!("Loading from code: {:?}", path);
        find_locs_path(path, &mut loaded, &mut locs)?;
    }
    Ok(locs)
}

/// attach the locations to the artifacts, returning locations that were not used.
pub fn attach_locs(artifacts: &mut Artifacts,
                   mut locs: HashMap<Name, Loc>)
                   -> Result<HashMap<Name, Loc>> {
    let mut dne: HashMap<Name, Loc> = HashMap::new();
    for (lname, loc) in locs.drain() {
        let artifact = match artifacts.get_mut(&lname) {
            Some(a) => a,
            None => {
                dne.insert(lname, loc);
                continue;
            }
        };
        if let Done::Defined(_) = artifact.done {
            return Err(ErrorKind::DoneTwice(lname.to_string()).into());
        }
        artifact.done = Done::Code(loc);
    }
    Ok(dne)
}

// Private Methods

fn find_locs_text(path: &Path, text: &str, locs: &mut HashMap<Name, Loc>) -> Result<()> {
    let mut line = 1;
    for cap in ART_LOC.captures_iter(text) {
        //debug_assert_eq!(cap.len(), 2);
        if let Some(m) = cap.get(1) {
            debug_assert!(cap.get(2).is_none());
            let name = Name::from_str(m.as_str()).expect("regex validated");
            let loc = Loc {
                path: path.to_path_buf(),
                line: line,
            };
            if let Some(first) = locs.insert(name, loc) {
                warn!("locations found twice. first: {}({}), \
                      second: {}({})",
                      first.path.display(),
                      first.line,
                      path.display(),
                      line);
            }
        } else {
            debug_assert!(cap.get(2).is_some());
            line += 1;
        }
    }
    Ok(())
}

/// given text, the path to the text, and the locations to add onto
/// extract all the locations from the text and return whether there
/// was an error
fn find_locs_file(path: &Path, locs: &mut HashMap<Name, Loc>) -> Result<()> {
    debug!("resolving locs at: {:?}", path);
    let mut text = String::new();
    let mut f = fs::File::open(path).chain_err(|| format!("opening file: {}", path.display()))?;
    if let Err(e) = f.read_to_string(&mut text) {
        if e.kind() == io::ErrorKind::InvalidData {
            warn!("non-utf8 file: {}", path.display());
            return Ok(());
        } else {
            Err(e).chain_err(|| format!("reading file: {}", path.display()))?;
        }
    }
    find_locs_text(path, &text, locs)
}

/// recursively find all locs given a directory
fn find_locs_dir(path: &PathBuf,
                 loaded: &mut HashSet<PathBuf>,
                 locs: &mut HashMap<Name, Loc>)
                 -> Result<()> {
    loaded.insert(path.to_path_buf());
    let read_dir = fs::read_dir(path).chain_err(|| format!("loading dir {}", path.display()))?;
    let mut dirs_to_load: Vec<PathBuf> = Vec::new(); // TODO: use references
    for entry in read_dir.filter_map(|e| e.ok()) {
        let fpath = entry.path();
        if loaded.contains(&fpath) {
            continue;
        }
        // don't parse .toml files for locations
        // TODO: make general instead
        match fpath.extension() {
            None => {}
            Some(ext) => {
                if ext == "toml" {
                    continue;
                }
            }
        }
        let ftype = entry.file_type().chain_err(|| format!("{}", fpath.display()))?;
        if ftype.is_dir() {
            dirs_to_load.push(fpath.clone());
        } else if ftype.is_file() {
            find_locs_file(&fpath, locs)?
        }
    }

    for d in dirs_to_load {
        find_locs_dir(&d, loaded, locs)?;
    }
    Ok(())
}

/// recursively find all locs given a directory
fn find_locs_path(path: &PathBuf,
                  loaded: &mut HashSet<PathBuf>,
                  locs: &mut HashMap<Name, Loc>)
                  -> Result<()> {
    let ty = path.metadata()
        .chain_err(|| format!("invalid path: {}", path.display()))?
        .file_type();
    if ty.is_file() {
        loaded.insert(path.to_path_buf());
        find_locs_file(path, locs)
    } else if ty.is_dir() {
        find_locs_dir(path, loaded, locs)
    } else {
        let msg = format!("invalid path: {}", path.display());
        Err(ErrorKind::PathNotFound(msg).into())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    pub const LOC_TEST: &'static str = "\
    $SPC-who
       #$SPC-what
     // $SPC-where
      //kjsdlfkjwe $TST-foo-what-where-2-b-3 kljasldkjf
    // $TST-dont-care
    /// $SPC-core-load-erro: <load file error>
    ";

    #[test]
    fn test_find_locs() {
        let mut locs: HashMap<Name, Loc> = HashMap::new();
        let path = PathBuf::from("hi/there");
        let loc_test = LOC_TEST.replace("$", "#");
        find_locs_text(&path, &loc_test, &mut locs).unwrap();
        // change: all locations are found
        assert!(locs.contains_key(&Name::from_str("TST-dont-care").unwrap()));

        let spc_who = locs.get(&Name::from_str("SPC-who").unwrap()).unwrap();
        let spc_what = locs.get(&Name::from_str("SPC-what").unwrap()).unwrap();
        let spc_where = locs.get(&Name::from_str("SPC-where").unwrap()).unwrap();
        let tst_long = locs.get(&Name::from_str("TST-foo-what-where-2-b-3").unwrap()).unwrap();
        let spc_error = locs.get(&Name::from_str("SPC-core-load-erro").unwrap()).unwrap();

        assert_eq!(spc_who.line, 1);
        assert_eq!(spc_what.line, 2);
        assert_eq!(spc_where.line, 3);
        assert_eq!(tst_long.line, 4);
        assert_eq!(spc_error.line, 6);
    }
}
