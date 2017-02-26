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
//! loadrs
//! loading of raw artifacts from files and text

use rustc_serialize::Decodable;

use toml::{Parser, Value, Table, Decoder};

use dev_prefix::*;
use super::types::*;
use super::utils;

use core::link;
use core::locs;

lazy_static!{
    pub static ref REPO_DIR: PathBuf = PathBuf::from(".art");
    pub static ref SETTINGS_PATH: PathBuf = REPO_DIR.join("settings.toml");
}

/// parse toml using a std error for this library
pub fn parse_toml(toml: &str) -> Result<Table> {
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

impl ProjectText {
    // TODO: making this parallel should be easy and dramatically improve performance:
    // - recursing through the directory, finding all the paths to files
    //     (and adding dirs to loaded_dirs)
    // - loading the files in parallel (IO bound)
    // - resolving all settings at the end
    /// recursively load the directory into text files, making sure
    /// not to load files that have already been loaded
    pub fn load(&mut self, load_dir: &Path, loaded_dirs: &mut HashSet<PathBuf>) -> Result<()> {
        loaded_dirs.insert(load_dir.to_path_buf());
        let mut dirs_to_load: Vec<PathBuf> = Vec::new();
        let dir_entries =
            fs::read_dir(load_dir).chain_err(|| format!(
            "could not get dir: {}", load_dir.display()))?;
        // just read text from all .toml files in the directory
        // and record which directories need to be loaded
        for entry in dir_entries.filter_map(|e| e.ok()) {
            let fpath = entry.path();
            let ftype = entry.file_type()
                .chain_err(|| format!("error reading type: {}", fpath.display()))?;
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
                let mut text = String::new();
                let mut fp = fs::File::open(&fpath).chain_err(|| format!(
                    "error opening: {}", fpath.display()))?;
                fp.read_to_string(&mut text)
                    .chain_err(|| format!("Error loading path {}", fpath.display()))?;
                self.files.insert(fpath.to_path_buf(), text);
            }
        }
        for dir in dirs_to_load {
            if !loaded_dirs.contains(dir.as_path()) {
                self.load(dir.as_path(), loaded_dirs)?;
            }
        }
        Ok(())
    }
}

impl Project {
    /// method to convert ProjectText -> Project
    /// Project may be extended by more than one ProjectText
    pub fn extend_text(&mut self, project_text: &ProjectText) -> Result<u64> {
        let mut count = 0;
        for (path, text) in &project_text.files {
            count += load_toml(path, text, self)?;
        }
        Ok(count)
    }
}

impl Settings {
    /// Load a settings object from a TOML Table
    pub fn from_table(tbl: &Table) -> Result<(RawSettings, Settings)> {
        let value = Value::Table(tbl.clone());
        let mut decoder = Decoder::new(value);
        let raw = RawSettings::decode(&mut decoder).chain_err(|| "invalid settings")?;

        if let Some(invalid) = decoder.toml {
            return Err(ErrorKind::InvalidSettings(format!("{:?}", invalid)).into());
        }

        fn to_paths(paths: &Option<Vec<String>>) -> VecDeque<PathBuf> {
            match *paths {
                Some(ref p) => {
                    p.iter()
                        .map(|p| PathBuf::from(utils::convert_path_str(p)))
                        .collect()
                }
                None => VecDeque::new(),
            }
        }
        let mut paths = to_paths(&raw.artifact_paths);
        let artifact_paths: HashSet<PathBuf> = paths.drain(0..).collect();
        let settings = Settings {
            artifact_paths: artifact_paths,
            code_paths: to_paths(&raw.code_paths),
            exclude_code_paths: to_paths(&raw.exclude_code_paths),
            additional_repos: to_paths(&raw.additional_repos),
        };

        Ok((raw, settings))
    }
}


impl Artifact {
    #[allow(should_implement_trait)]
    /// from_str is mosty used to make testing and one-off development easier
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
            text: raw.text.unwrap_or_default(),
            partof: try!(ArtNames::from_str(&raw.partof.unwrap_or_default())),
            loc: None,

            // calculated vars
            parts: HashSet::new(),
            completed: -1.0,
            tested: -1.0,
        })
    }
}

/// Given text load the artifacts
pub fn load_toml(path: &Path, text: &str, project: &mut Project) -> Result<u64> {
    // parse the text
    let table = parse_toml(text)?;
    let mut num_loaded: u64 = 0;
    project.files.insert(path.to_path_buf());

    for (name, value) in &table {
        let aname = ArtName::from_str(name)?;
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
        let artifact = Artifact::from_table(&aname, path, art_tbl)?;
        project.artifacts.insert(Arc::new(aname), artifact);
        num_loaded += 1;
    }
    Ok(num_loaded)
}

///// load settings from a path recursively
//pub fn load_repo_settings(path: &Path) -> Result<VecDeque<(RawSettings, Settings)>> {
//    let mut f = fs::File::open(path)?;
//    let mut text = String::new();
//    f.read_to_string(&mut text)?;

//    let tbl = try!(parse_toml(&text));
//    let (raw, settings) = Settings::from_table(&tbl)?;

//    let mut out = VecDeque::new();
//    for r in &settings.additional_repos {
//        out.extend(load_repo_settings(r)?);
//    }
//    out.push_front((raw, settings));
//    Ok(out)
//}


pub fn resolve_settings_paths(repo: &Path, settings: &mut Settings) -> Result<()> {
    let mut vars: HashMap<String, String> = HashMap::new();
    // TODO: for full windows compatibility you will probably want to support OsStr
    // here... I just don't want to yet
    let settings_path = repo.join(SETTINGS_PATH.as_path());
    {
        let cwd = repo.join(REPO_DIR.as_path());
        let cwd_str = utils::get_path_str(&cwd)?;
        vars.insert(CWD_VAR.to_string(), cwd_str.to_string());
        vars.insert(REPO_VAR.to_string(), utils::get_path_str(repo)?.to_string());
    }

    {
        // push resolved artifact_paths
        let mut paths = HashSet::new();
        for p in &settings.artifact_paths {
            let p = utils::do_strfmt(utils::get_path_str(p)?, &vars, &settings_path)?;
            let p = utils::canonicalize(Path::new(&p)).chain_err(
                || format!("could not find artifact_path: {}", p))?;
            paths.insert(p);
        }
        settings.artifact_paths = paths;
    }

    {
        // push resolved code_paths
        let mut paths = VecDeque::new();
        for p in &settings.code_paths {
            let p = try!(utils::do_strfmt(utils::get_path_str(p)?, &vars, &settings_path));
            let p = utils::canonicalize(Path::new(&p)).chain_err(
                || format!("could not find code_path: {}", p))?;
            paths.push_back(p);
        }
        settings.code_paths = paths;
    }

    {
        // push resolved exclude_code_paths
        let mut paths = VecDeque::new();
        for p in &settings.exclude_code_paths {
            let p = try!(utils::do_strfmt(utils::get_path_str(p)?, &vars, &settings_path));
            // if an exclude path doesn't exist that's fine
            let p = match utils::canonicalize(Path::new(&p)) {
                Ok(p) => p,
                Err(_) => {
                    info!("could not find exclude path: {}", p);
                    continue;
                }
            };
            paths.push_back(p);
        }
        settings.exclude_code_paths = paths;
    }
    Ok(())
}

/// load settings from a repo
pub fn load_settings(repo: &Path) -> Result<Settings> {
    let settings_path = repo.join(SETTINGS_PATH.as_path());
    let mut text = String::new();
    let mut f = fs::File::open(&settings_path).chain_err(|| format!(
        "error opening settings: {}", settings_path.display()))?;
    f.read_to_string(&mut text)
        .chain_err(|| format!("error reading settings: {}", settings_path.display()))?;

    let tbl = parse_toml(&text).chain_err(|| format!(
        "error parsing settings: {}", settings_path.display()))?;
    let (_, mut settings) = Settings::from_table(&tbl)?;

    resolve_settings_paths(repo, &mut settings)?;

    Ok(settings)
}

pub fn process_project(project: &mut Project) -> Result<()> {
    let locs = locs::find_locs(&project.settings)?;
    project.dne_locs = locs::attach_locs(&mut project.artifacts, locs);
    link::do_links(&mut project.artifacts)?;
    Ok(())
}

/// load a processed project
pub fn load_project(repo: &Path) -> Result<Project> {
    let settings = load_settings(repo)?;

    let mut project_text = ProjectText::default();
    let mut loaded_dirs: HashSet<PathBuf> = HashSet::new(); // see SPC-load-dir, RSK-2-load-loop
    loaded_dirs.insert(repo.join(REPO_DIR.as_path()));

    for dir in &settings.artifact_paths {
        if loaded_dirs.contains(dir) {
            warn!("artifact_paths tried to load a directory twice: {}",
                  dir.display());
            continue;
        }
        loaded_dirs.insert(dir.to_path_buf());
        project_text.load(dir.as_path(), &mut loaded_dirs)?;
    }

    let mut project = Project::default();
    project.settings = settings.clone();
    project.extend_text(&project_text)?;

    process_project(&mut project)?;

    project.origin = repo.to_path_buf();

    Ok(project)
}
