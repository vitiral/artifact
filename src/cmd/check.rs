use super::types::*;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("check")
        .about("check for any errors in the project")
        .settings(&[AS::DeriveDisplayOrder, AS::ColoredHelp])
}

pub fn do_check<W: Write>(w: &mut W,
                           cwd: &Path,
                           artifacts: &Artifacts,
                           dne_locs: &HashMap<ArtName, Loc>,
                           settings: &Settings) -> i32 {
    let mut error: i32 = 0;
    // display invalid partof names and locations
    // partof: #SPC-check-load
    let mut invalid_partof = ArtNames::new();

    fn paint_it<W: Write>(w: &mut W, settings: &Settings, msg: &str) {
        if settings.color {
            write!(w, "{}", Red.paint(msg)).unwrap();
        } else {
            write!(w, "{}", msg).unwrap();
        }
    }

    let mut displayed_header = false;
    for (name, artifact) in artifacts.iter() {
        invalid_partof.clear();
        for p in artifact.partof.iter() {
            if !artifacts.contains_key(p) {
                invalid_partof.insert(p.clone());
            }
        }
        if invalid_partof.len() > 0 {
            error = 1;
            let mut msg = String::new();
            if !displayed_header {
                write!(msg, "\n# Found partof names that do not exist:\n").unwrap();
                displayed_header = true;
            }
            write!(msg, "    {} [{}]: {:?}\n",
                   name, utils::relative_path(&artifact.path, cwd).display(),
                   invalid_partof).unwrap();
            paint_it(w, settings, &msg);
        }
    }

    // display invalid locations
    if dne_locs.len() > 0 {
        error = 1;
        // reorganize them by file
        let mut invalid_locs: HashMap<PathBuf, Vec<(ArtName, Loc)>> = HashMap::new();
        for (name, loc) in dne_locs {
            if !invalid_locs.contains_key(&loc.path) {
                invalid_locs.insert(loc.path.clone(), Vec::new());
            }
            invalid_locs.get_mut(&loc.path).unwrap().push((name.clone(), loc.clone()));
        }
        let header = "\n# Found implementation links in the code that do not exist:\n";
        if settings.color {
            write!(w, "{}", Red.bold().paint(header)).unwrap();
        } else {
            write!(w, "{}", header).unwrap();
        }
        let mut invalid_locs: Vec<(PathBuf, Vec<(ArtName, Loc)>)> = Vec::from_iter(
            invalid_locs.drain());
        invalid_locs.sort_by(|a, b| a.0.cmp(&b.0));
        for (path, mut locs) in invalid_locs.drain(0..) {
            // sort by where they appear in the file
            let mut pathstr = String::new();
            write!(pathstr, "    {}:\n", utils::relative_path(&path, cwd).display()).unwrap();
            paint_it(w, settings, &pathstr);
            locs.sort_by(|a, b| a.1.line_col.cmp(&b.1.line_col));
            for (name, loc) in locs {
                let mut loc_str = String::new();
                write!(loc_str, "    - ({}:{})", loc.line_col.0, loc.line_col.1).unwrap();
                paint_it(w, settings, &loc_str);
                write!(w, " {}\n", name).unwrap();
            }
            write!(w, "\n").unwrap();
        }
    }
    // find hanging artifacts
    // partof: #SPC-check-hanging

    fn partof_types(a: &Artifact, types: &HashSet<ArtType>) -> bool {
        for p in a.partof.iter() {
            if types.contains(&p.get_type()) {
                return true;
            }
        }
        false
    }
    let rsk_spc_types = HashSet::from_iter(vec![ArtType::RSK, ArtType::SPC]);
    let req_types = HashSet::from_iter(vec![ArtType::REQ]);

    let mut hanging: Vec<(ArtNameRc, &Path)> = Vec::new();
    for (name, artifact) in artifacts.iter() {
        let ty = name.get_type();
        if (ty != ArtType::REQ) && !artifact.is_parent() && !name.is_root()
                && name.parent().unwrap().is_root() {
            if match ty {
                ArtType::TST => !partof_types(artifact, &rsk_spc_types),
                ArtType::SPC | ArtType::RSK=> !partof_types(artifact, &req_types),
                _ => unreachable!(),
            } {
                hanging.push((name.clone(), &artifact.path));
            }
        }
    }
    hanging.sort_by(|a, b| a.1.cmp(b.1));
    if hanging.len() > 0 {
        error = 1;
        let msg = "Hanging artifacts found (top-level but not partof a higher type):\n";
        paint_it(w, settings, msg);
        for (h, p) in hanging {
            let mut msg = String::new();
            write!(msg, "    {:<30}: {}\n", utils::relative_path(p, cwd).display(), h);
            paint_it(w, settings, &msg);
        }
    }

    if error == 0 {
        let mut msg = String::new();
        write!(msg, "rst check: no errors found in {}\n", cwd.display());
        if settings.color {
            write!(w, "{}", Green.paint(msg)).unwrap();
        } else {
            write!(w, "{}", msg).unwrap();
        }
    }
    error
}
