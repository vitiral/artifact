use core::types::*;

/// #SPC-core-load-settings-resolve:<resolve all informaiton related to settings>
pub fn resolve_settings(settings: &mut Settings,
                        repo_map: &mut HashMap<PathBuf, PathBuf>,
                        loaded_settings: &Vec<(PathBuf, Settings)>)
                        -> LoadResult<()> {
    // first pull out all of the repo_names
    for ps in loaded_settings.iter() {
        let ref s: &Settings = &ps.1;
        for rn in &s.repo_names {
            settings.repo_names.insert(rn.clone());
        }
    }

    // now resolve all path names
    let mut vars: HashMap<String, String> = HashMap::new();
    for ps in loaded_settings.iter() {
        let ref settings_item: &Settings = &ps.1;

        let fpath = ps.0.clone();
        let cwd = fpath.parent().unwrap();
        let cwd_str = try!(get_path_str(cwd));

        // TODO: for full windows compatibility you will probably want to support OsStr
        // here... I just don't want to
        // [#SPC-core-settings-vars]
        vars.insert("cwd".to_string(), cwd_str.to_string());
        try!(find_and_insert_repo(cwd, repo_map, &settings.repo_names));
        let repo = repo_map.get(cwd).unwrap();
        vars.insert("repo".to_string(), try!(get_path_str(repo.as_path())).to_string());

        // push resolved paths
        for p in settings_item.paths.iter() {
            let p = try!(do_strfmt(p.to_str().unwrap(), &vars, &fpath));
            settings.paths.push_back(PathBuf::from(p));
        }

        // TODO: it is possible to be able to use all global variables in code_paths
        // push resolved code_paths
        for p in settings_item.code_paths.iter() {
            let p = try!(do_strfmt(p.to_str().unwrap(), &vars, &fpath));
            settings.code_paths.push_back(PathBuf::from(p));
        }

        // push resolved exclude_code_paths
        for p in settings_item.exclude_code_paths.iter() {
            let p = try!(do_strfmt(p.to_str().unwrap(), &vars, &fpath));
            settings.exclude_code_paths.push_back(PathBuf::from(p));
        }
    }
    Ok(())
}
