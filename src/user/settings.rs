//! loading settings

use toml::{Value, Table, Decoder};
use rustc_serialize::Decodable;

use dev_prefix::*;
use types::*;
use user::types::*;
use user::utils;
use utils::parse_toml;

// Public Methods

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
    let (_, mut settings) = from_table(&tbl)?;

    resolve_settings_paths(repo, &mut settings)?;

    Ok(settings)
}


// Public For Tests

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

// Private Methods

/// resolve the variables in the settings paths
fn resolve_settings_paths(repo: &Path, settings: &mut Settings) -> Result<()> {
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

#[cfg(test)]
mod tests {
    use dev_prefix::*;
    use super::*;
    use test_data;
    use utils;

    #[test]
    fn test_settings() {
        let tbl = utils::parse_toml(test_data::TOML_SETTINGS).unwrap();
        let (_, set) = from_table(&tbl).unwrap();
        assert!(set.artifact_paths ==
                HashSet::from_iter(vec![PathBuf::from("{cwd}/test"),
                                        PathBuf::from("{repo}/test")]));
        assert!(set.code_paths ==
                VecDeque::from_iter(vec![PathBuf::from("{cwd}/src"),
                                         PathBuf::from("{repo}/src2")]));

        let toml_invalid = r#"
        artifact_paths = ['hi']
        paths = ['invalid']
        "#;
        let tbl = utils::parse_toml(toml_invalid).unwrap();

        assert!(from_table(&tbl).is_err());
    }
}
