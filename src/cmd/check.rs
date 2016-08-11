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
    fn paint_it_bold<W: Write>(w: &mut W, settings: &Settings, msg: &str) {
        if settings.color {
            write!(w, "{}", Red.bold().paint(msg)).unwrap();
        } else {
            write!(w, "{}", msg).unwrap();
        }
    }

    // display artifacts with invalid partof names
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
                displayed_header = true;
                paint_it_bold(w, settings, "\nFound partof names that do not exist:\n");
            }
            write!(msg, "    {} [{}]: {:?}\n",
                   name, utils::relative_path(&artifact.path, cwd).display(),
                   invalid_partof).unwrap();
            paint_it(w, settings, &msg);
        }
    }

    // display unresolvable partof names
    let unresolved: Vec<(ArtNameRc, &Artifact)> = Vec::from_iter(
        artifacts.iter()
            .filter(|a| a.1.completed < 0. || a.1.tested < 0.)
            .map(|n| (n.0.clone(), n.1)));
    let unknown_names: HashSet<ArtNameRc> = HashSet::from_iter(
        unresolved.iter()
            .map(|u| u.0.clone()));

    if unresolved.len() > 0 {
        error = 1;
        let mut unresolved_partof: HashMap<ArtNameRc, HashSet<ArtNameRc>> = HashMap::new();
        for &(ref name, artifact) in unresolved.iter() {
            let partof: HashSet<_> = artifact.partof
                .iter()
                .filter(|n| !artifacts.contains_key(n.as_ref())
                        || unknown_names.contains(n.as_ref()))
                .cloned()
                .collect();
            unresolved_partof.insert(name.clone(), partof);
        }

        // reduce unresolved partof to only items that have at least one value
        let mut resolved_names: HashSet<ArtNameRc> = HashSet::new();
        let mut remove_names = Vec::new();
        let mut just_resolved = Vec::new();
        loop {
            just_resolved.clear();
            let mut did_something = false;
            for (name, partof) in unresolved_partof.iter_mut() {
                // find names in partof that have no unresolved partofs and remove them
                remove_names.clear();
                for p in partof.iter() {
                    if resolved_names.contains(p) {
                        remove_names.push(p.clone());
                    }
                }
                if remove_names.len() > 0 {
                    did_something = true;
                }
                for p in remove_names.iter() {
                    partof.remove(p);
                }

                // if this artifact has no unresolved partofs, then it is considered resolved
                if partof.len() == 0 {
                    resolved_names.insert(name.clone());
                    just_resolved.push(name.clone());
                }
            }
            if just_resolved.len() > 0 {
                did_something = true;
            }
            for r in just_resolved.iter() {
                unresolved_partof.remove(r);
            }
            if !did_something {
                break;
            }
        }
        paint_it_bold(w, settings, "\nArtifacts partof contains at least one recursive reference:\n");
        let mut unresolved_partof: Vec<_> = unresolved_partof
            .drain()
            .map(|mut v| (v.0, v.1.drain().collect::<Vec<_>>()))
            .collect();
        unresolved_partof.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, partof) in unresolved_partof.drain(0..) {
            let mut msg = String::new();
            write!(msg, "    {:<30}: {:?}\n",
                   name.to_string(), partof).unwrap();
            write!(w, "{}", msg).unwrap();
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
        let header = "\nFound implementation links in the code that do not exist:\n";
        paint_it_bold(w, settings, header);
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
        let msg = "\nHanging artifacts found (top-level but not partof a higher type):\n";
        paint_it_bold(w, settings, msg);
        for (h, p) in hanging {
            let mut msg = String::new();
            write!(msg, "    {:<30}: {}\n", utils::relative_path(p, cwd).display(), h).unwrap();
            write!(w, "{}", msg).unwrap();
        }
    }

    if error == 0 {
        let mut msg = String::new();
        write!(msg, "rst check: no errors found in {}\n", cwd.display()).unwrap();
        if settings.color {
            write!(w, "{}", Green.paint(msg)).unwrap();
        } else {
            write!(w, "{}", msg).unwrap();
        }
    } else {
        write!(w, "\n").unwrap();
    }
    error
}
