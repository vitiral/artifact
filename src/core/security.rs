use dev_prefix::*;
use core::prefix::*;
use core;


/// validate that all files that could be affected by
/// the project are within the repo and the `settings.artifact_paths`
/// partof: #SPC-security-gen
pub fn validate(repo: &Path, project: &Project) -> Result<()> {
    let mut files: HashSet<&PathBuf> = HashSet::new();
    files.extend(project.artifacts.values().map(|a| &a.path));
    files.extend(project.files.iter());
    files.extend(project.settings_map.keys());
    files.extend(project.raw_settings_map.keys());
    files.extend(project.variables_map.keys());
    files.extend(project.repo_map.keys());

    // PARENT_PATH is never written to, so ignore
    files.remove(&*core::types::PARENT_PATH);

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
        if !project.settings.artifact_paths.iter().any(|p| f.starts_with(p)) {
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
