//! loadrs
//! loading of raw artifacts from files and text

use super::types::*;
use super::vars;

use super::utils;

use toml::{Parser, Value, Table};

lazy_static!{
    pub static ref ARTIFACT_ATTRS: HashSet<String> = HashSet::from_iter(
        ["disabled", "text", "partof"].iter().map(|s| s.to_string()));
    pub static ref SETTINGS_ATTRS: HashSet<String> = HashSet::from_iter(
        ["disabled", "artifact_paths",
         "code_paths", "exclude_code_paths"].iter().map(|s| s.to_string()));
}

macro_rules! get_attr {
    ($tbl: expr, $attr: expr, $default: expr, $ty: ident) => {
        match $tbl.get($attr) {
            // If the value is in the table, return the value
            Some(&Value::$ty(ref v)) => Some(v.clone()),
            // otherwise return the default
            None => Some($default.clone()),
            // If it's the wrong type, return None (Err)
            _ => None,
        }
    }
}

/// only one type is in an array, so make this custom
pub fn get_vecstr(tbl: &Table, attr: &str, default: &Vec<String>)
              -> Option<Vec<String>> {
    match tbl.get(attr) {
        // if the value is in the table, try to get it's elements
        Some(&Value::Array(ref a)) => {
            let mut out: Vec<String> = Vec::with_capacity(a.len());
            for v in a {
                match v {
                    &Value::String(ref s) => out.push(s.clone()),
                    _ => return None,  // error: invalid type
                }
            }
            Some(out)
        }
        None => Some(default.clone()), // value doesn't exist, return default
        _ => None,  // error: invalid type
    }
}

/// #TST-core-load-table-check:<check the type to make sure it matches>
macro_rules! check_type {
    ($value: expr, $attr: expr, $name: expr) => {
        match $value {
            Some(v) => v,
            None => {
                let mut msg = Vec::new();
                write!(&mut msg, "{} has invalid attribute: {}", $name, $attr).unwrap();
                return Err(LoadError::new(String::from_utf8(msg).unwrap()));
            }
        }
    }
}

impl Settings {
    /// Load a settings object from a TOML Table
    /// partof: #SPC-settings-load
    pub fn from_table(tbl: &Table) -> LoadResult<Settings> {
        let invalid_attrs: Vec<_> = tbl.keys()
            .filter(|k| !SETTINGS_ATTRS.contains(k.as_str())).collect();
        if invalid_attrs.len() > 0 {
            let mut msg = String::new();
            write!(msg, "invalid attributes in settings: {:?}", invalid_attrs).unwrap();
            return Err(LoadError::new(msg));
        }

        let df_vec = Vec::new();
        let str_paths: Vec<String> = check_type!(
            get_vecstr(tbl, "artifact_paths", &df_vec), "artifact_paths", "settings");
        let code_paths: Vec<String> = check_type!(
            get_vecstr(tbl, "code_paths", &df_vec), "code_paths", "settings");
        let exclude_code_paths: Vec<String> = check_type!(
            get_vecstr(tbl, "exclude_code_paths", &df_vec), "exclude_code_paths", "settings");
        Ok(Settings {
            disabled: check_type!(get_attr!(tbl, "disabled", false, Boolean),
                                  "disabled", "settings"),
            paths: str_paths.iter().map(|s| PathBuf::from(s)).collect(),
            code_paths: code_paths.iter().map(|s| PathBuf::from(s)).collect(),
            exclude_code_paths: exclude_code_paths.iter().map(|s| PathBuf::from(s)).collect(),
            color: true,
        })
    }
}


/// parse toml using a std error for this library
fn parse_toml(toml: &str) -> LoadResult<Table> {
    let mut parser = Parser::new(toml);
    match parser.parse() {
        Some(table) => Ok(table),
        None => {
            let mut msg = String::new();
            for e in &parser.errors {
                let (line, col) = parser.to_linecol(e.lo);
                write!(msg, "[{}:{}] {}, ", line, col, e.desc).unwrap();
            }
            Err(LoadError::new(msg))
        },
    }
}

impl Artifact {
    /// from_str is mosty used to make testing and one-off development easier
    pub fn from_str(toml: &str) -> LoadResult<(ArtName, Artifact)> {
        let table = try!(parse_toml(toml));
        if table.len() != 1 {
            return Err(LoadError::new("must contain a single table".to_string()));
        }
        let (name, value) = table.iter().next().unwrap();
        let name = try!(ArtName::from_str(name));
        let value = match value {
            &Value::Table(ref t) => t,
            _ => return Err(LoadError::new("must contain a single table".to_string())),
        };
        let artifact = try!(Artifact::from_table(&name, &Path::new("from_str"), value));
        Ok((name, artifact))
    }

    /// Create an artifact object from a toml Table
    /// partof: #SPC-artifact-load
    fn from_table(name: &ArtName, path: &Path, tbl: &Table) -> LoadResult<Artifact> {
        let df_str = "".to_string();
        let invalid_attrs: Vec<_> = tbl.keys()
            .filter(|k| !ARTIFACT_ATTRS.contains(k.as_str())).collect();
        if invalid_attrs.len() > 0 {
            let mut msg = String::new();
            write!(msg, "{} has invalid attributes: {:?}", name, invalid_attrs).unwrap();
            return Err(LoadError::new(msg));
        }

        // partf: #SPC-artifact-partof-1: explicitly set artifact
        let partof_str = check_type!(get_attr!(tbl, "partof", df_str, String),
                                    "partof", name);
        Ok(Artifact{
            // loaded vars
            ty: name.get_type(),
            path: path.to_path_buf(),
            text: check_type!(get_attr!(tbl, "text", df_str, String),
                              "text", name),
            partof: try!(ArtNames::from_str(&partof_str)),
            loc: None,

            // calculated vars
            parts: HashSet::new(),
            completed: -1.0,
            tested: -1.0,
        })
    }
}

/// Load artifacts and settings from a toml Table
pub fn load_file_table(file_table: &mut Table, path: &Path,
                       artifacts: &mut Artifacts,
                       settings: &mut Vec<(PathBuf, Settings)>,
                       variables: &mut Vec<(PathBuf, Variables)>)
                       -> LoadResult<u64> {
    let mut msg: Vec<u8> = Vec::new();
    let mut num_loaded: u64 = 0;

    match file_table.remove("settings") {
        Some(Value::Table(t)) => {
            let lset = try!(Settings::from_table(&t));
            // [#SPC-core-settings-disabled]
            if lset.disabled {
                return Ok(0);
            }
            settings.push((path.to_path_buf(), lset));
        }
        None => {},
        _ => return Err(LoadError::new("settings must be a Table".to_string())),
    }

    match file_table.remove("globals") {
        Some(Value::Table(t)) => {
            let mut lvars = Variables::new();
            for (k, v) in t {
                if vars::DEFAULT_GLOBALS.contains(k.as_str()) {
                    return Err(LoadError::new("cannot use variables: repo, cwd".to_string()));
                }
                lvars.insert(k.clone(), match v {
                    Value::String(s) => s.to_string(),
                    _ => return Err(LoadError::new(
                        k.to_string() + " global var must be of type str")),
                });
            }
            variables.push((path.to_path_buf(), lvars));
        }
        None => {},
        _ => return Err(LoadError::new("globals must be a Table".to_string())),
    }

    for (name, value) in file_table.iter() {
        let aname = try!(ArtName::from_str(name));
        // get the artifact table
        let art_tbl: &Table = match value {
            &Value::Table(ref t) => t,
            _ => {
                write!(&mut msg, "All top-level values must be a table: {}", name).unwrap();
                return Err(LoadError::new(String::from_utf8(msg).unwrap()));
            }
        };
        // check for overlap
        if let Some(overlap) = artifacts.get(&aname) {
            write!(&mut msg, "Overlapping key found <{}> other key at: {}",
                name, overlap.path.display()).unwrap();
            return Err(LoadError::new(String::from_utf8(msg).unwrap()));
        }
        // [#SPC-core-artifact-attrs-disabled]
        if check_type!(get_attr!(art_tbl, "disabled", false, Boolean),
                       "disabled", name) {
            continue
        }
        let artifact = try!(Artifact::from_table(&aname, path, art_tbl));
        artifacts.insert(aname, artifact);
        num_loaded += 1;
    }
    return Ok(num_loaded);
}

pub fn load_toml_simple(text: &str) -> Artifacts {
    let mut artifacts = Artifacts::new();
    let mut settings: Vec<(PathBuf, Settings)> = Vec::new();
    let mut variables: Vec<(PathBuf, Variables)> = Vec::new();
    let path = PathBuf::from("test");
    load_toml(&path, text, &mut artifacts, &mut settings, &mut variables).unwrap();
    artifacts
}

/// Given text load the artifacts
pub fn load_toml(path: &Path, text: &str,
                 artifacts: &mut Artifacts,
                 settings: &mut Vec<(PathBuf, Settings)>,
                 variables: &mut Vec<(PathBuf, Variables)>)
                 -> LoadResult<u64> {
    // parse the text
    let mut table = try!(parse_toml(text));
    load_file_table(&mut table, path, artifacts, settings, variables)
}

/// given a file path load the artifacts
///
/// #SPC-core-load-file
pub fn load_file(path: &Path,
                 artifacts: &mut Artifacts,
                 settings: &mut Vec<(PathBuf, Settings)>,
                 variables: &mut Vec<(PathBuf, Variables)>)
                 -> LoadResult<u64> {
    // let mut text: Vec<u8> = Vec::new();

    // read the text
    let mut text = String::new();
    let mut fp = fs::File::open(path).unwrap();
    try!(fp.read_to_string(&mut text).or_else(
        |err| {
            let mut msg = String::new();
            write!(msg, "Error loading path {:?}: {}", path, err).unwrap();
            Err(LoadError::new(msg))
         }));
    load_toml(path, &text, artifacts, settings, variables)
}

/// recursively load a directory, ensuring that sub-directories don't get
/// double loaded
/// partof: #SPC-load-dir
pub fn load_dir(path: &Path,
                loaded_dirs: &mut HashSet<PathBuf>,
                artifacts: &mut Artifacts,
                settings: &mut Vec<(PathBuf, Settings)>,
                variables: &mut Vec<(PathBuf, Variables)>)
                -> LoadResult<u64> {
    loaded_dirs.insert(path.to_path_buf());
    // TDOO: if load_path.is_dir()
    let mut num_loaded: u64 = 0;
    let mut error = false;
    // for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
    let mut dirs_to_load: Vec<PathBuf> = Vec::new(); // TODO: references should be possible here...
    let read_dir = match fs::read_dir(path) {
        Ok(d) => d,
        Err(err) => return Err(LoadError::new("E001: ".to_string() + &err.to_string())),
    };
    // process all the files in the directory. Process directories later
    for entry in read_dir.filter_map(|e| e.ok()) {
        let fpath = entry.path();
        let ftype = match entry.file_type() {
            Ok(f) => f,
            Err(err) => {
                error!("while loading from <{}>: {}", fpath.display(), err);
                error = true;
                continue;
            }
        };
        if ftype.is_dir() {
            dirs_to_load.push(fpath.clone());
        } else if ftype.is_file() {
            let ext = match fpath.extension() {
                None => continue,
                Some(ext) => ext,
            };
            if ext != "rsk" { // only load rsk files
                continue
            }
            match load_file(fpath.as_path(), artifacts, settings, variables) {
                Ok(n) => num_loaded += n,
                Err(err) => {
                    error!("while loading from <{}>: {}", fpath.display(), err);
                    error = true;
                }
            };
        }
    };
    // don't recurse if no .rsk files are found
    if num_loaded > 0 {
        for dir in dirs_to_load {
            if loaded_dirs.contains(dir.as_path()) {
                continue;
            }
            match load_dir(dir.as_path(), loaded_dirs, artifacts, settings, variables) {
                Ok(n) => num_loaded += n,
                Err(_) => error = true,
            }
        }
    }
    if error {
        // [#SPC-core-load-error-file-return]
        return Err(LoadError::new("ERROR: some files failed to load".to_string()));
    } else {
        Ok(num_loaded)
    }
}

/// push settings found (loaded_settings) into a main settings object
/// repo_map is a pre-compiled hashset mapping dirs->repo_path (for performance)
/// partof: #SPC-settings-resolve
pub fn resolve_settings(settings: &mut Settings,
                        repo_map: &mut HashMap<PathBuf, PathBuf>,
                        loaded_settings: &Vec<(PathBuf, Settings)>)
                        -> LoadResult<()> {
    // now resolve all path names
    let mut vars: HashMap<String, String> = HashMap::new();
    for ps in loaded_settings.iter() {
        let ref settings_item: &Settings = &ps.1;

        let fpath = ps.0.clone();
        let cwd = fpath.parent().unwrap();
        let cwd_str = try!(utils::get_path_str(cwd));

        // TODO: for full windows compatibility you will probably want to support OsStr
        // here... I just don't want to
        vars.insert("cwd".to_string(), cwd_str.to_string());
        try!(utils::find_and_insert_repo(cwd, repo_map));
        let repo = repo_map.get(cwd).unwrap();
        vars.insert("repo".to_string(), try!(utils::get_path_str(repo.as_path())).to_string());

        // push resolved paths
        for p in settings_item.paths.iter() {
            let p = try!(utils::do_strfmt(p.to_str().unwrap(), &vars, &fpath));
            settings.paths.push_back(PathBuf::from(p));
        }

        // TODO: it is possible to be able to use all global variables in code_paths
        //    but then it must be done in a separate step
        // push resolved code_paths
        for p in settings_item.code_paths.iter() {
            let p = try!(utils::do_strfmt(p.to_str().unwrap(), &vars, &fpath));
            settings.code_paths.push_back(PathBuf::from(p));
        }

        // push resolved exclude_code_paths
        for p in settings_item.exclude_code_paths.iter() {
            let p = try!(utils::do_strfmt(p.to_str().unwrap(), &vars, &fpath));
            settings.exclude_code_paths.push_back(PathBuf::from(p));
        }
    }
    Ok(())
}

/// given a valid path, load all paths given by the settings recursively
/// partof: #SPC-load-raw
pub fn load_raw(path: &Path)
                -> LoadResult<(Artifacts, Settings,
                               Vec<(PathBuf, Variables)>,
                               HashMap<PathBuf, PathBuf>)> {
    let mut artifacts = Artifacts::new();
    let mut settings = Settings::new();
    let mut loaded_dirs: HashSet<PathBuf> = HashSet::new(); // see SPC-load-dir, RSK-2-load-loop
    let mut loaded_settings: Vec<(PathBuf, Settings)> = Vec::new();
    let mut loaded_vars: Vec<(PathBuf, Variables)> = Vec::new();
    // repo_map maps directories to their found base-repositories
    let mut repo_map: HashMap<PathBuf, PathBuf> = HashMap::new();
    let mut msg = String::new();

    info!("Loading artifact files:");
    if path.is_file() {
        try!(load_file(path, &mut artifacts, &mut loaded_settings,
                       &mut loaded_vars));
        try!(resolve_settings(&mut settings, &mut repo_map, &loaded_settings));
    } else if path.is_dir() {
        settings.paths.push_back(path.to_path_buf());
    } else {
        return Err(LoadError::new("File is not valid type: ".to_string() +
                                  path.to_string_lossy().as_ref()));
    }

    // #SPC-core-load-parts-1:<load and validate all paths recursively>
    while settings.paths.len() > 0 {
        let dir = settings.paths.pop_front().unwrap(); // it has len, it better pop!
        if loaded_dirs.contains(&dir) {
            continue
        }
        debug!("Loading artifacts: {:?}", dir);
        loaded_settings.clear();
        loaded_dirs.insert(dir.to_path_buf());
        match load_dir(dir.as_path(), &mut loaded_dirs,
                       &mut artifacts, &mut loaded_settings,
                       &mut loaded_vars) {
            Ok(n) => n,
            Err(err) => {
                write!(msg, "Error loading <{}>: {}", dir.to_string_lossy().as_ref(), err).unwrap();
                return Err(LoadError::new(msg));
            }
        };

        // resolve the project-level settings after each directory is recursively loaded
        // so that artifact_paths can be resolved
        // see: SPC-settings-resolve
        try!(resolve_settings(&mut settings, &mut repo_map, &loaded_settings));
    }

    Ok((artifacts, settings, loaded_vars, repo_map))
}
