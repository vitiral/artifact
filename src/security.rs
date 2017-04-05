use dev_prefix::*;
use types::*;


/// validate that all files that could be affected by
/// the project are within the repo and the `settings.artifact_paths`
/// partof: #SPC-security
pub fn validate(repo: &Path, project: &Project) -> Result<()> {
    let mut files: HashSet<&PathBuf> = HashSet::new();
    files.extend(project.artifacts.values().map(|a| &a.path));
    files.extend(project.files.iter());
    files.extend(project.repo_map.keys());

    // PARENT_PATH is never written to, so ignore
    files.remove(&*PARENT_PATH);

    for f in files {
        if !f.is_absolute() {
            let msg = format!("{} is not an absolute path", f.display());
            return Err(ErrorKind::Internal(msg).into());
        }
        // only allow files that are in the cwd repo
        if !f.starts_with(repo) {
            let msg = format!("{} is not a subdir of cwd repo {}",
                              f.display(),
                              repo.display());
            return Err(ErrorKind::Security(msg).into());
        }
        // only allow files that are in the artifact_paths
        if !project.settings
                .artifact_paths
                .iter()
                .any(|p| f.starts_with(p)) {
            let msg = format!("{} is not a subdir of any artifact_paths {:?}",
                              f.display(),
                              project.settings.artifact_paths);
            return Err(ErrorKind::Security(msg).into());
        }
        // files that do not already exist are invalid
        if !f.exists() {
            let msg = format!("{} does not already exist and cannot be created here",
                              f.display());
            return Err(ErrorKind::Security(msg).into());
        }
    }
    Ok(())
}

pub fn validate_settings(repo: &Path, settings: &Settings) -> Result<()> {
    println!("repo={:?}, artifact_paths={:?}", repo, settings.artifact_paths);
    if settings.artifact_paths
            .iter()
            .any(|p| !p.starts_with(repo)) {
        //TODO improve message
        let msg = format!("artifact_paths invalid");
        Err(ErrorKind::Security(msg).into())
    }
    else {
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use test_data;
    use utils;
    use user;

    // TODO: not sure if this is even needed anymore... will check again
    // later
    //#[test]
    ///// make sure that artifacts which are loaded "out of bounds"
    ///// don't make it past the security checker
    ///// partof: #TST-security-gen
    //fn test_bounds_checker() {
    //    let design = test_data::TINVALID_BOUNDS.join("repo").join("design");
    //    let repo = utils::find_repo(&design).unwrap();
    //    match user::load_repo(&repo) {
    //        Err(e) => {
    //            match *e.kind() {
    //                ErrorKind::Security(_) => { [> expected <] }
    //                _ => panic!("unexpected error: {:?}", e.display()),
    //            }
    //        }
    //        Ok(_) => panic!("fmt accidentally suceeded -- may need to reset with git"),
    //    }
    //    //let project = user::load_repo(&repo).unwrap();
    //    //let req_bounds = NameRc::from_str("REQ-bounds").unwrap();
    //    //assert!(project.artifacts.contains_key(&req_bounds));
    //    //assert_eq!(project.artifacts[&req_bounds].path,
    //    //           test_data::TINVALID_BOUNDS.join("out_bounds.toml"));
    //    //assert!(validate(&repo, &project).is_err());
    //}

    #[test]
    fn test_security() {
        let design = test_data::TINVALID_BOUNDS.join("repo").join("design");
        let repo = utils::find_repo(&design).unwrap();
        match user::load_repo(&repo) {
            Err(e) => {
                match *e.kind() {
                    ErrorKind::Security(_) => { /* expected */ }
                    _ => panic!("unexpected error: {:?}", e.display()),
                }
            }
            Ok(_) => panic!("fmt accidentally suceeded -- may need to reset with git"),
        }
    }
}
