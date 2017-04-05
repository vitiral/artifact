//! loading settings

use toml::{Value, Table, Decoder};
use rustc_serialize::Decodable;

use dev_prefix::*;
use types::*;
use user::types::*;
use user::utils;
use utils::{parse_toml, canonicalize};

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

    fn to_paths(paths: &Option<Vec<String>>) -> HashSet<PathBuf> {
        match *paths {
            Some(ref p) => p.iter().map(|p| PathBuf::from(utils::convert_path_str(p))).collect(),
            None => HashSet::new(),
        }
    }
    let settings = Settings {
        artifact_paths: to_paths(&raw.artifact_paths),
        exclude_artifact_paths: to_paths(&raw.exclude_artifact_paths),
        code_paths: to_paths(&raw.code_paths),
        exclude_code_paths: to_paths(&raw.exclude_code_paths),
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

    fn resolve_paths(ignore_errors: bool,
                     name: &str,
                     paths: &HashSet<PathBuf>,
                     vars: &HashMap<String, String>,
                     settings_path: &Path)
                     -> Result<HashSet<PathBuf>> {
        // push resolved exclude_artifact_paths
        let mut out = HashSet::new();
        for p in paths {
            let p =
                utils::do_strfmt(utils::get_path_str(p)?, vars, settings_path).chain_err(|| {
                                   format!("replacing variables failed at {}: {}",
                                           name,
                                           p.display())
                               })?;
            // if an exclude path doesn't exist that's fine
            let p = match canonicalize(Path::new(&p)) {
                Ok(p) => p,
                Err(err) => {
                    if ignore_errors {
                        debug!("could not find {} path: {}", name, p);
                        continue;
                    } else {
                        return Err(err).chain_err(|| format!("could not find {}: {}", name, p));
                    }
                }
            };
            out.insert(p);
        }
        Ok(out)
    }
    settings.artifact_paths = resolve_paths(false,
                                            "artifact_paths",
                                            &settings.artifact_paths,
                                            &vars,
                                            &settings_path)?;
    settings.exclude_artifact_paths = resolve_paths(true,
                                                    "exclude_artifact_paths",
                                                    &settings.exclude_artifact_paths,
                                                    &vars,
                                                    &settings_path)?;
    settings.code_paths = resolve_paths(false,
                                        "code_paths",
                                        &settings.code_paths,
                                        &vars,
                                        &settings_path)?;
    settings.exclude_code_paths = resolve_paths(true,
                                                "exclude_code_paths",
                                                &settings.exclude_code_paths,
                                                &vars,
                                                &settings_path)?;
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
                HashSet::from_iter(vec![PathBuf::from("{cwd}/src"), PathBuf::from("{repo}/src2")]));

        let toml_invalid = r#"
        artifact_paths = ['hi']
        paths = ['invalid']
        "#;
        let tbl = utils::parse_toml(toml_invalid).unwrap();

        assert!(from_table(&tbl).is_err());
    }
}
