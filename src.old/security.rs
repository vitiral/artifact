use dev_prefix::*;
use types::*;


/// validate that all files that could be affected by
/// the project are within the repo and the `settings.artifact_paths`
/// partof: #SPC-security
pub fn validate(repo: &Path, project: &Project) -> Result<()> {
    let mut files: HashSet<&PathBuf> = HashSet::new();
    files.extend(project.artifacts.values().map(|a| &a.def));
    files.extend(project.files.iter());
    files.extend(project.repo_map.keys());

    for f in files {
        if !f.is_absolute() {
            let msg = format!("{} is not an absolute path", f.display());
            return Err(ErrorKind::Internal(msg).into());
        }
        // only allow files that are in the cwd repo
        if !f.starts_with(repo) {
            let msg = format!(
                "{} is not a subdir of cwd repo {}",
                f.display(),
                repo.display()
            );
            return Err(ErrorKind::Security(msg).into());
        }
        // only allow files that are in the artifact_paths
        if !project
            .settings
            .artifact_paths
            .iter()
            .any(|p| f.starts_with(p))
        {
            let msg = format!(
                "{} is not a subdir of any artifact_paths {:?}",
                f.display(),
                project.settings.artifact_paths
            );
            return Err(ErrorKind::Security(msg).into());
        }
        // files that do not already exist are invalid
        if !f.exists() {
            let msg = format!(
                "{} does not already exist and cannot be created here",
                f.display()
            );
            return Err(ErrorKind::Security(msg).into());
        }
    }
    Ok(())
}

pub fn validate_settings(repo: &Path, settings: &Settings) -> Result<()> {
    if settings.artifact_paths.iter().any(|p| !p.starts_with(repo)) {
        //TODO improve message
        let msg = "`artifact_paths` invalid".to_string();
        Err(ErrorKind::Security(msg).into())
    } else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_data;
    use utils;
    use user;
    use tempdir;
    use fs_extra::dir;

    #[test]
    /// partof: #TST-security-bounds
    fn test_bounds() {
        test_bounds_init();
        test_bounds_edit();
    }

    fn test_bounds_init() {
        let design = test_data::TINVALID_BOUNDS.join("repo").join("design");
        let repo = utils::find_repo(&design).unwrap();
        match user::load_repo(&repo) {
            Err(e) => {
                match *e.kind() {
                    ErrorKind::Security(_) => { /* expected */ }
                    _ => panic!("Unexpected error: {:?}", e.display()),
                }
            }
            Ok(_) => panic!(
                "CRITICAL: Fmt suceeded when it should not have -- may need to reset with git"
            ),
        }
    }

    fn test_bounds_edit() {
        let tmpdir = tempdir::TempDir::new("artifact").unwrap();
        let writedir = tmpdir.path();
        dir::copy(
            &test_data::TSIMPLE_DIR.as_path(),
            &writedir,
            &dir::CopyOptions::new(),
        ).unwrap();
        let simple = writedir.join("simple");
        let repo = utils::find_repo(&simple).unwrap();
        let mut project = user::load_repo(&repo).unwrap();
        let (name, mut art) = Artifact::from_str("[SPC-out_bounds]\n").unwrap();
        art.def = writedir.to_path_buf();

        project.artifacts.insert(name, art);

        assert!(validate(&repo, &project).is_err());
    }
}
