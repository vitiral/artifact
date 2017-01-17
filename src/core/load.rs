/*  rst: the requirements tracking tool made for developers
 * Copyright (C) 2016  Garrett Berg <@vitiral, vitiral@gmail.com>
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
//! loadrs
//! loading of raw artifacts from files and text

use rustc_serialize::Decodable;
use toml::{Parser, Value, Table, Decoder};

use dev_prefix::*;
use super::types::*;
use super::vars;
use super::utils;

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
pub fn get_vecstr(tbl: &Table, attr: &str, default: &[String]) -> Option<Vec<String>> {
    match tbl.get(attr) {
        // if the value is in the table, try to get it's elements
        Some(&Value::Array(ref a)) => {
            let mut out: Vec<String> = Vec::with_capacity(a.len());
            for v in a {
                match *v {
                    Value::String(ref s) => out.push(s.clone()),
                    _ => return None,  // error: invalid type
                }
            }
            Some(out)
        }
        None => Some(Vec::from(default)), // value doesn't exist, return default
        _ => None,  // error: invalid type
    }
}

/// check the type to make sure it matches
macro_rules! check_type {
    ($value: expr, $attr: expr, $name: expr) => {
        match $value {
            Some(v) => v,
            None => return Err(ErrorKind::InvalidAttr(
                $name.to_string(), $attr.to_string()).into()),
        }
    }
}

#[cfg(not(windows))]
fn get_color(raw: &RawSettings) -> bool {
    raw.color.unwrap_or(true)
}

#[cfg(windows)]
/// color always disabled for windows
fn get_color(raw: &RawSettings) -> bool {
    false
}

impl Settings {
    /// Load a settings object from a TOML Table
    /// partof: #SPC-settings-load
    pub fn from_table(tbl: &Table) -> Result<(RawSettings, Settings)> {
        let value = Value::Table(tbl.clone());
        let mut decoder = Decoder::new(value);
        let raw = RawSettings::decode(&mut decoder).chain_err(|| "invalid settings")?;

        if let Some(invalid) = decoder.toml {
            return Err(ErrorKind::InvalidSettings(format!("{:?}", invalid)).into());
        }

        fn to_paths(paths: &Option<Vec<String>>) -> VecDeque<PathBuf> {
            match *paths {
                Some(ref p) => p.iter().map(PathBuf::from).collect(),
                None => VecDeque::new(),
            }
        }
        let settings = Settings {
            disabled: raw.disabled.unwrap_or_default(),
            paths: to_paths(&raw.artifact_paths),
            code_paths: to_paths(&raw.code_paths),
            exclude_code_paths: to_paths(&raw.exclude_code_paths),
            color: get_color(&raw),
        };

        Ok((raw, settings))
    }
}


/// parse toml using a std error for this library
fn parse_toml(toml: &str) -> Result<Table> {
    let mut parser = Parser::new(toml);
    match parser.parse() {
        Some(table) => Ok(table),
        None => {
            let mut locs = String::new();
            for e in &parser.errors {
                let (line, col) = parser.to_linecol(e.lo);
                write!(locs, "[{}:{}] {}, ", line, col, e.desc).unwrap();
            }
            Err(ErrorKind::TomlParse(locs).into())
        }
    }
}

impl Artifact {
    /// from_str is mosty used to make testing and one-off development easier
    #[allow(should_implement_trait)]
    pub fn from_str(toml: &str) -> Result<(ArtNameRc, Artifact)> {
        let table = try!(parse_toml(toml));
        if table.len() != 1 {
            return Err(ErrorKind::Load("must contain a single table".to_string()).into());
        }
        let (name, value) = table.iter().next().unwrap();
        let name = try!(ArtName::from_str(name));
        let value = match *value {
            Value::Table(ref t) => t,
            _ => return Err(ErrorKind::Load("must contain a single table".to_string()).into()),
        };
        let artifact = try!(Artifact::from_table(&name, &Path::new("from_str"), value));
        Ok((Arc::new(name), artifact))
    }

    /// Create an artifact object from a toml Table
    /// partof: #SPC-artifact-load
    fn from_table(name: &ArtName, path: &Path, tbl: &Table) -> Result<Artifact> {
        let value = Value::Table(tbl.clone());
        let mut decoder = Decoder::new(value);
        let raw = match RawArtifact::decode(&mut decoder) {
            Ok(v) => v,
            Err(e) => {
                return Err(ErrorKind::InvalidArtifact(name.to_string(), e.to_string()).into())
            }
        };

        if let Some(invalid) = decoder.toml {
            return Err(ErrorKind::InvalidArtifact(name.to_string(),
                                                  format!("invalid attrs: {}", invalid))
                .into());
        }

        Ok(Artifact {
            path: path.to_path_buf(),
            text: Text::new(&raw.text.unwrap_or_default()),
            partof: try!(ArtNames::from_str(&raw.partof.unwrap_or_default())),
            loc: None,

            // calculated vars
            parts: HashSet::new(),
            completed: -1.0,
            tested: -1.0,
        })
    }
}

/// Load artifacts and settings from a toml Table
pub fn load_file_table(file_table: &mut Table, path: &Path, project: &mut Project) -> Result<u64> {
    let mut msg: Vec<u8> = Vec::new();
    let mut num_loaded: u64 = 0;

    match file_table.remove("settings") {
        Some(Value::Table(t)) => {
            let (raw, settings) = try!(Settings::from_table(&t));
            project.raw_settings_map.insert(path.to_path_buf(), raw);
            if settings.disabled {
                return Ok(0);
            }
            project.settings_map.insert(path.to_path_buf(), settings);
        }
        None => {}
        _ => return Err(ErrorKind::InvalidSettings("settings must be a Table".to_string()).into()),
    }

    match file_table.remove("globals") {
        Some(Value::Table(t)) => {
            let mut variables = Variables::new();
            for (k, v) in t {
                if vars::DEFAULT_GLOBALS.contains(k.as_str()) {
                    return Err(ErrorKind::InvalidVariable("cannot use variables: repo, cwd"
                            .to_string())
                        .into());
                }
                let value = match v {
                    Value::String(s) => s.to_string(),
                    _ => {
                        return Err(ErrorKind::InvalidVariable(format!("{} global var must be of \
                                                                       type str",
                                                                      k))
                            .into())
                    }
                };
                variables.insert(k.clone(), value);
            }
            project.variables_map.insert(path.to_path_buf(), variables);
        }
        None => {}
        _ => return Err(ErrorKind::InvalidVariable("globals must be a Table".to_string()).into()),
    }

    for (name, value) in file_table.iter() {
        let aname = try!(ArtName::from_str(name));
        // get the artifact table
        let art_tbl: &Table = match *value {
            Value::Table(ref t) => t,
            _ => {
                write!(&mut msg, "All top-level values must be a table: {}", name).unwrap();
                return Err(ErrorKind::Load(String::from_utf8(msg).unwrap()).into());
            }
        };
        // check for overlap
        if let Some(overlap) = project.artifacts.get(&aname) {
            write!(&mut msg,
                   "Overlapping key found <{}> other key at: {}",
                   name,
                   overlap.path.display())
                .unwrap();
            return Err(ErrorKind::Load(String::from_utf8(msg).unwrap()).into());
        }
        if check_type!(get_attr!(art_tbl, "disabled", false, Boolean),
                       "disabled",
                       name) {
            continue;
        }
        let artifact = try!(Artifact::from_table(&aname, path, art_tbl));
        project.artifacts.insert(Arc::new(aname), artifact);
        num_loaded += 1;
    }
    Ok(num_loaded)
}

pub fn load_toml_simple(text: &str) -> Artifacts {
    let mut project = Project::default();
    let path = PathBuf::from("test");
    load_toml(&path, text, &mut project).unwrap();
    project.artifacts
}

/// Given text load the artifacts
pub fn load_toml(path: &Path, text: &str, project: &mut Project) -> Result<u64> {
    // parse the text
    let mut table = try!(parse_toml(text));
    load_file_table(&mut table, path, project)
}

/// given a file path load the artifacts
///
pub fn load_file(path: &Path, project: &mut Project) -> Result<u64> {
    // let mut text: Vec<u8> = Vec::new();

    // read the text
    let mut text = String::new();
    let mut fp = fs::File::open(path).unwrap();
    fp.read_to_string(&mut text)
        .chain_err(|| format!("Error loading path {}", path.display()))?;
    load_toml(path, &text, project)
}

/// recursively load a directory, ensuring that sub-directories don't get
/// double loaded
// TODO: making this parallel should be easy and dramatically improve performance:
// - recursing through the directory, finding all the paths to files
//     (and adding dirs to loaded_dirs)
// - loading the files in parallel (IO bound)
// - resolving all settings at the end
/// partof: #SPC-load-dir
pub fn load_dir(path: &Path,
                loaded_dirs: &mut HashSet<PathBuf>,
                project: &mut Project)
                -> Result<u64> {
    loaded_dirs.insert(path.to_path_buf());
    // TDOO: if load_path.is_dir()
    let mut num_loaded: u64 = 0;
    let mut error = false;
    // for entry in WalkDir::new(&path).into_iter().filter_map(|e| e.ok()) {
    let mut dirs_to_load: Vec<PathBuf> = Vec::new();
    let read_dir =
        fs::read_dir(path).chain_err(|| format!("could not get dir: {}", path.display()))?;
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
            if ext != "toml" {
                // only load toml files
                continue;
            }
            match load_file(fpath.as_path(), project) {
                Ok(n) => num_loaded += n,
                Err(err) => {
                    error!("while loading from <{}>: {}", fpath.display(), err);
                    error = true;
                }
            };
            project.files.insert(fpath.to_path_buf());
        }
    }
    // only recurse if .toml files with data were found
    if num_loaded > 0 {
        for dir in dirs_to_load {
            if loaded_dirs.contains(dir.as_path()) {
                continue;
            }
            match load_dir(dir.as_path(), loaded_dirs, project) {
                Ok(n) => num_loaded += n,
                Err(_) => error = true,
            }
        }
    }
    if error {
        Err(ErrorKind::Load("some files failed to load".to_string()).into())
    } else {
        Ok(num_loaded)
    }
}

/// push settings found (`loaded_settings`) into a main settings object
/// `repo_map` is a pre-compiled hashset mapping `dirs->repo_path` (for performance)
/// partof: #SPC-settings-resolve
pub fn resolve_settings(project: &mut Project) -> Result<()> {
    // now resolve all path names
    let mut vars: HashMap<String, String> = HashMap::new();
    for ps in &project.settings_map {
        let file_settings: &Settings = ps.1;

        let fpath = ps.0.clone();
        let cwd = fpath.parent().unwrap();
        let cwd_str = try!(utils::get_path_str(cwd));

        // TODO: for full windows compatibility you will probably want to support OsStr
        // here... I just don't want to yet
        vars.insert("cwd".to_string(), cwd_str.to_string());
        try!(utils::find_and_insert_repo(cwd, &mut project.repo_map));
        let repo = &project.repo_map[cwd];
        vars.insert("repo".to_string(),
                    try!(utils::get_path_str(repo.as_path())).to_string());

        // push resolved paths
        for p in &file_settings.paths {
            let p = try!(utils::do_strfmt(p.to_str().unwrap(), &vars, &fpath));
            project.settings.paths.push_back(PathBuf::from(p));
        }

        // TODO: it is possible to be able to use all global variables in code_paths
        //    but then it must be done in a separate step
        // push resolved code_paths
        for p in &file_settings.code_paths {
            let p = try!(utils::do_strfmt(p.to_str().unwrap(), &vars, &fpath));
            project.settings.code_paths.push_back(PathBuf::from(p));
        }

        // push resolved exclude_code_paths
        for p in &file_settings.exclude_code_paths {
            let p = try!(utils::do_strfmt(p.to_str().unwrap(), &vars, &fpath));
            project.settings.exclude_code_paths.push_back(PathBuf::from(p));
        }
    }
    Ok(())
}

fn extend_settings(full: &mut HashMap<PathBuf, Settings>,
                   dir: &HashMap<PathBuf, Settings>)
                   -> Result<()> {
    for (p, s) in dir.iter() {
        if full.insert(p.clone(), s.clone()).is_some() {
            return Err(ErrorKind::Load(format!("Internal Error: file loaded twice: {}",
                                               p.display()))
                .into());
        }
    }
    Ok(())
}


/// given a valid path, load all paths given by the settings recursively
/// partof: #SPC-load-raw
#[allow(type_complexity)]  // TODO: probably remove this
pub fn load_raw(path: &Path) -> Result<Project> {
    let mut project = Project::default();
    let mut loaded_dirs: HashSet<PathBuf> = HashSet::new(); // see SPC-load-dir, RSK-2-load-loop
    let mut full_settings_map: HashMap<PathBuf, Settings> = HashMap::new();

    info!("Loading artifact files:");
    if path.is_file() {
        try!(load_file(path, &mut project));
        try!(extend_settings(&mut full_settings_map, &project.settings_map));
        try!(resolve_settings(&mut project));
        project.files.insert(path.to_path_buf());
    } else if path.is_dir() {
        project.settings.paths.push_back(path.to_path_buf());
    } else {
        return Err(ErrorKind::Load("File is not valid type: ".to_string() +
                                   path.to_string_lossy().as_ref())
            .into());
    }

    while !project.settings.paths.is_empty() {
        let dir = project.settings.paths.pop_front().unwrap(); // it has len, it better pop!
        if loaded_dirs.contains(&dir) {
            continue;
        }
        debug!("Loading artifacts: {:?}", dir);
        // we only need to resolve settings one dir at a time
        project.settings_map.clear();
        loaded_dirs.insert(dir.to_path_buf());
        match load_dir(dir.as_path(), &mut loaded_dirs, &mut project) {
            Ok(n) => n,
            Err(err) => {
                return Err(ErrorKind::Load(format!("Error loading <{}>: {}",
                                                   dir.to_string_lossy().as_ref(),
                                                   err))
                    .into())
            }
        };

        // resolve the project-level settings after each directory is recursively loaded
        // so that we can find new artifact_paths
        // see: SPC-settings-resolve
        try!(extend_settings(&mut full_settings_map, &project.settings_map));
        try!(resolve_settings(&mut project));
    }

    project.variables = try!(vars::resolve_loaded_vars(&project.variables_map,
                                                       &mut project.repo_map));
    project.settings_map = full_settings_map;
    Ok(project)
}
