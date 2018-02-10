use dev_prefix::*;
use std::hash::Hash;
use types::*;

const ART_LOC_NAME_POS: usize = 1;
const ART_LOC_SUB_POS: usize = 2;
const ART_LOC_NEWLINE_POS: usize = 3;

lazy_static!{
    pub static ref ART_LOC: Regex = Regex::new(
        &format!(r"(?i)(?:#({})(\.[{}]+)?)|(\n)", NAME_VALID_STR, NAME_VALID_CHARS!())).unwrap();
}

// Public Methods

/// search through the `code_paths` in settings to find all valid locs
pub fn find_locs(settings: &Settings) -> Result<(HashMap<Name, Loc>, HashMap<SubName, Loc>)> {
    info!("parsing code files for artifacts...");
    let mut locs: HashMap<Name, Loc> = HashMap::new();
    let mut sublocs: HashMap<SubName, Loc> = HashMap::new();
    let mut loaded: HashSet<PathBuf> =
        HashSet::from_iter(settings.exclude_code_paths.iter().map(|p| p.to_path_buf()));
    debug!("excluded code paths: {:?}", loaded);
    for path in &settings.code_paths {
        if loaded.contains(path) {
            continue;
        }
        debug!("Loading from code: {:?}", path);
        find_locs_path(path, &mut loaded, &mut locs, &mut sublocs)?;
    }
    Ok((locs, sublocs))
}

#[allow(map_entry)]
/// attach the locations to the artifacts, returning locations that were not used.
pub fn attach_locs(
    artifacts: &mut Artifacts,
    mut locs: HashMap<Name, Loc>,
    mut sublocs: HashMap<SubName, Loc>,
) -> Result<(HashMap<Name, Loc>, HashMap<SubName, Loc>)> {
    // Merge the locs and sublocs into the FullLocs object
    let mut full_locs: HashMap<NameRc, FullLocs> = HashMap::new();
    let mut dne: HashMap<Name, Loc> = HashMap::new();
    let mut dne_sublocs: HashMap<SubName, Loc> = HashMap::new();

    for (lname, loc) in locs.drain() {
        if !artifacts.contains_key(&lname) {
            dne.insert(lname, loc);
            continue;
        }
        if !full_locs.contains_key(&lname) {
            full_locs.insert(Arc::new(lname.clone()), FullLocs::empty());
        }
        let full = full_locs.get_mut(&lname).unwrap();
        full.root = Some(loc);
    }

    for (lname, loc) in sublocs.drain() {
        match artifacts.get(&lname.name) {
            Some(a) => {
                if !a.subnames.contains(&lname) {
                    // the parent exists but the subname does not
                    dne_sublocs.insert(lname, loc);
                    continue;
                }
            }
            None => {
                // not even the parent name exists!
                dne_sublocs.insert(lname, loc);
                continue;
            }
        }
        if !full_locs.contains_key(&lname.name) {
            full_locs.insert(lname.name.clone(), FullLocs::empty());
        }
        let full = full_locs.get_mut(&lname.name).unwrap();
        full.sublocs.insert(lname, loc);
    }

    for (lname, loc) in full_locs.drain() {
        let artifact = artifacts.get_mut(&lname).expect("checked above");
        if let Done::Defined(_) = artifact.done {
            return Err(ErrorKind::DoneTwice(lname.to_string()).into());
        }
        artifact.done = Done::Code(loc);
    }

    Ok((dne, dne_sublocs))
}

// Private Methods

fn find_locs_text(
    path: &Path,
    text: &str,
    locs: &mut HashMap<Name, Loc>,
    sublocs: &mut HashMap<SubName, Loc>,
) -> Result<()> {
    let mut line = 1;

    for cap in ART_LOC.captures_iter(text) {
        //debug_assert_eq!(cap.len(), ART_LOC_NEWLINE_POS);
        if let Some(m) = cap.get(ART_LOC_NAME_POS) {
            debug_assert!(cap.get(ART_LOC_NEWLINE_POS).is_none());
            let name = Name::from_str(m.as_str()).expect("regex validated");
            let loc = Loc {
                path: path.to_path_buf(),
                line: line,
            };

            /// Quick generic function for inserting with warnings
            fn insert_loc<N: Clone + Debug + Eq + Hash>(
                hmap: &mut HashMap<N, Loc>,
                name: &N,
                loc: &Loc,
            ) {
                if let Some(first) = hmap.insert(name.clone(), loc.clone()) {
                    warn!(
                        "locations for {:?} found twice. first: {}({}), \
                         second: {}({})",
                        name,
                        first.path.display(),
                        first.line,
                        loc.path.display(),
                        loc.line
                    );
                }
            }

            if let Some(m) = cap.get(ART_LOC_SUB_POS) {
                let sub = m.as_str().split_at(1).1.to_string(); // strip the leading '.'
                let sub = SubName::from_parts(Arc::new(name), sub);
                insert_loc(sublocs, &sub, &loc);
            } else {
                insert_loc(locs, &name, &loc);
            }
        } else {
            debug_assert!(cap.get(ART_LOC_NEWLINE_POS).is_some());
            line += 1;
        }
    }
    Ok(())
}

/// given text, the path to the text, and the locations to add onto
/// extract all the locations from the text and return whether there
/// was an error
fn find_locs_file(
    path: &Path,
    locs: &mut HashMap<Name, Loc>,
    sublocs: &mut HashMap<SubName, Loc>,
) -> Result<()> {
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
    find_locs_text(path, &text, locs, sublocs)
}

/// recursively find all locs given a directory
fn find_locs_dir(
    path: &PathBuf,
    loaded: &mut HashSet<PathBuf>,
    locs: &mut HashMap<Name, Loc>,
    sublocs: &mut HashMap<SubName, Loc>,
) -> Result<()> {
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
            Some(ext) => if ext == "toml" {
                continue;
            },
        }
        let ftype = entry
            .file_type()
            .chain_err(|| format!("{}", fpath.display()))?;
        if ftype.is_dir() {
            dirs_to_load.push(fpath.clone());
        } else if ftype.is_file() {
            find_locs_file(&fpath, locs, sublocs)?
        }
    }

    for d in dirs_to_load {
        find_locs_dir(&d, loaded, locs, sublocs)?;
    }
    Ok(())
}

/// recursively find all locs given a directory
fn find_locs_path(
    path: &PathBuf,
    loaded: &mut HashSet<PathBuf>,
    locs: &mut HashMap<Name, Loc>,
    sublocs: &mut HashMap<SubName, Loc>,
) -> Result<()> {
    let ty = path.metadata()
        .chain_err(|| format!("invalid path: {}", path.display()))?
        .file_type();
    if ty.is_file() {
        loaded.insert(path.to_path_buf());
        find_locs_file(path, locs, sublocs)
    } else if ty.is_dir() {
        find_locs_dir(path, loaded, locs, sublocs)
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
    /// $SPC-who.subpart
    /// $SPC-core-load-erro.OTHER_subpart
    ";

    #[test]
    fn test_find_locs() {
        let mut locs: HashMap<Name, Loc> = HashMap::new();
        let mut sublocs: HashMap<SubName, Loc> = HashMap::new();
        let path = PathBuf::from("hi/there");
        let loc_test = LOC_TEST.replace("$", "#");
        find_locs_text(&path, &loc_test, &mut locs, &mut sublocs).unwrap();

        assert_eq!(locs.len(), 6);
        assert_eq!(sublocs.len(), 2);

        let spc_who_name = Name::from_str("SPC-who").unwrap();
        let spc_error_name = Name::from_str("SPC-core-load-erro").unwrap();

        // change: all locations are found
        assert!(locs.contains_key(&Name::from_str("TST-dont-care").unwrap()));

        let spc_who = locs.get(&spc_who_name).unwrap();
        let spc_what = locs.get(&Name::from_str("SPC-what").unwrap()).unwrap();
        let spc_where = locs.get(&Name::from_str("SPC-where").unwrap()).unwrap();
        let tst_long = locs.get(&Name::from_str("TST-foo-what-where-2-b-3").unwrap())
            .unwrap();
        let spc_error = locs.get(&spc_error_name).unwrap();

        assert_eq!(spc_who.line, 1);
        assert_eq!(spc_what.line, 2);
        assert_eq!(spc_where.line, 3);
        assert_eq!(tst_long.line, 4);
        assert_eq!(spc_error.line, 6);

        // Assert subparts
        let spc_who_sub = sublocs
            .get(&SubName::from_parts(
                Arc::new(spc_who_name.clone()),
                "subpart".into(),
            ))
            .unwrap();
        let spc_error_sub = sublocs
            .get(&SubName::from_parts(
                Arc::new(spc_error_name.clone()),
                // note: case doesn't matter
                "other_subpart".into(),
            ))
            .unwrap();

        assert_eq!(spc_who_sub.line, 7);
        assert_eq!(spc_error_sub.line, 8);
    }
}
