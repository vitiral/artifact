//! loading settings

use toml;

use dev_prefix::*;
use types::*;
use user::types::*;
use user::utils;
use utils::canonicalize;

// Public Methods

/// load settings from a repo
pub fn load_settings(repo: &Path) -> Result<Settings> {
    let settings_path = repo.join(SETTINGS_PATH.as_path());
    let mut text = String::new();
    let mut f = fs::File::open(&settings_path).chain_err(|| {
        format!("Error opening settings: {}", settings_path.display())
    })?;
    f.read_to_string(&mut text).chain_err(|| {
        format!("Error reading settings: {}", settings_path.display())
    })?;
    from_text(repo, &text)
}


// Public For Tests

pub fn from_text(repo: &Path, text: &str) -> Result<Settings> {
    let raw: RawSettings = toml::from_str(text)?;
    let mut settings = from_raw(&raw)?;
    resolve_settings_paths(repo, &mut settings)?;
    Ok(settings)
}

/// Load a settings object from a TOML Table
pub fn from_raw(raw: &RawSettings) -> Result<Settings> {
    fn to_paths(paths: &Option<Vec<String>>) -> HashSet<PathBuf> {
        match *paths {
            Some(ref p) => p.iter()
                .map(|p| PathBuf::from(utils::convert_path_str(p)))
                .collect(),
            None => HashSet::new(),
        }
    }
    let file_type = match raw.file_type.as_ref() {
        None => FileType::Toml,
        Some(s) => match s.as_str() {
            "toml" => FileType::Toml,
            "markdown" => FileType::Markdown,
            _ => bail!(ErrorKind::InvalidSettings(
                format!("unknown file_type: {}", s)
            )),
        },
    };
    let settings = Settings {
        artifact_paths: to_paths(&raw.artifact_paths),
        exclude_artifact_paths: to_paths(&raw.exclude_artifact_paths),
        code_paths: to_paths(&raw.code_paths),
        exclude_code_paths: to_paths(&raw.exclude_code_paths),
        file_type: file_type,
    };

    Ok(settings)
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

    fn resolve_paths(
        ignore_errors: bool,
        name: &str,
        paths: &HashSet<PathBuf>,
        vars: &HashMap<String, String>,
        settings_path: &Path,
    ) -> Result<HashSet<PathBuf>> {
        // push resolved exclude_artifact_paths
        let mut out = HashSet::new();
        for p in paths {
            let p = utils::do_strfmt(utils::get_path_str(p)?, vars, settings_path).chain_err(|| {
                format!("Replacing variables failed at {}: {}", name, p.display())
            })?;
            // if an exclude path doesn't exist that's fine
            let p = match canonicalize(Path::new(&p)) {
                Ok(p) => p,
                Err(err) => if ignore_errors {
                    debug!("Could not find {} path: {}", name, p);
                    continue;
                } else {
                    return Err(err).chain_err(|| format!("Could not find {}: {}", name, p));
                },
            };
            out.insert(p);
        }
        Ok(out)
    }
    settings.artifact_paths = resolve_paths(
        false,
        "artifact_paths",
        &settings.artifact_paths,
        &vars,
        &settings_path,
    )?;
    settings.exclude_artifact_paths = resolve_paths(
        true,
        "exclude_artifact_paths",
        &settings.exclude_artifact_paths,
        &vars,
        &settings_path,
    )?;
    settings.code_paths = resolve_paths(
        false,
        "code_paths",
        &settings.code_paths,
        &vars,
        &settings_path,
    )?;
    settings.exclude_code_paths = resolve_paths(
        true,
        "exclude_code_paths",
        &settings.exclude_code_paths,
        &vars,
        &settings_path,
    )?;
    let artifact_intersection: Vec<_> = settings
        .artifact_paths
        .intersection(&settings.exclude_artifact_paths)
        .collect();
    if !artifact_intersection.is_empty() {
        let msg = format!(
            "Some items in artifact_paths are also in exclude_artifact_paths: {:?}",
            artifact_intersection,
        );
        return Err(ErrorKind::InvalidSettings(msg).into());
    }
    let code_intersection: Vec<_> = settings
        .code_paths
        .intersection(&settings.exclude_code_paths)
        .collect();
    if !code_intersection.is_empty() {
        let msg = format!(
            "Some items in code_paths are also in exclude_code_paths: {:?}",
            code_intersection,
        );
        return Err(ErrorKind::InvalidSettings(msg).into());
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use dev_prefix::*;
    use super::*;
    use test_data;

    #[test]
    fn test_settings() {
        let raw: RawSettings = toml::from_str(test_data::TOML_SETTINGS).unwrap();
        let set = from_raw(&raw).unwrap();
        assert!(
            set.artifact_paths ==
                HashSet::from_iter(vec![
                    PathBuf::from("{cwd}/test"),
                    PathBuf::from("{repo}/test"),
                ])
        );
        assert!(
            set.code_paths ==
                HashSet::from_iter(vec![
                    PathBuf::from("{cwd}/src"),
                    PathBuf::from("{repo}/src2"),
                ])
        );

        let toml_invalid = r#"
        artifact_paths = ['hi']
        paths = ['invalid']
        "#;
        assert!(toml::from_str::<RawSettings>(toml_invalid).is_err());
    }
}
