use super::types::*;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("status")
        .about("get the status of the project")
        .settings(&[AS::DeriveDisplayOrder, AS::ColoredHelp])
}

pub fn do_status<W: Write>(w: &mut W,
                           cwd: &Path,
                           artifacts: &Artifacts,
                           dne_locs: &HashMap<ArtName, Loc>)
                           -> io::Result<()> {
    let mut error = false;
    // display invalid partof names and locations
    // partof: #SPC-status-load
    let mut invalid_partof = ArtNames::new();
    for (name, artifact) in artifacts.iter() {
        invalid_partof.clear();
        for p in artifact.partof.iter() {
            if !artifacts.contains_key(p) {
                invalid_partof.insert(p.clone());
            }
        }
        if invalid_partof.len() > 0 {
            if !error {
                try!(write!(w, "\n# Found partof names that do not exist:\n"));
                error = true;
            }
            write!(w, "    {} [{}]: {:?}\n",
                   name, utils::relative_path(&artifact.path, cwd).display(),
                   invalid_partof) ;
        }
    }

    // display invalid locations
    if dne_locs.len() > 0 {
        // reorganize them by file
        let mut invalid_locs: HashMap<PathBuf, Vec<(ArtName, Loc)>> = HashMap::new();
        for (name, loc) in dne_locs {
            if !invalid_locs.contains_key(&loc.path) {
                invalid_locs.insert(loc.path.clone(), Vec::new());
            }
            invalid_locs.get_mut(&loc.path).unwrap().push((name.clone(), loc.clone()));
        }
        try!(write!(w, "\n# Found implementation links in the code that do not exist:\n"));
        let mut invalid_locs: Vec<(PathBuf, Vec<(ArtName, Loc)>)> = Vec::from_iter(
            invalid_locs.drain());
        invalid_locs.sort_by(|a, b| a.0.cmp(&b.0));
        for (path, mut locs) in invalid_locs.drain(0..) {
            try!(write!(w, "    {}: ", utils::relative_path(&path, cwd).display()));
            locs.sort_by(|a, b| a.0.cmp(&b.0));
            for (name, loc) in locs {
                try!(write!(w, "{}({}:{}) ", name, loc.line_col.0, loc.line_col.1));
            }
            try!(write!(w, "\n"));
        }
    }
    Ok(())
}
