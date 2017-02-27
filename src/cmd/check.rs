/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2017  Garrett Berg <@vitiral, vitiral@gmail.com>
 *
 * This program is free software: you can redistribute it and/or modify
 * it under the terms of the Lesser GNU General Public License as published
 * by the Free Software Foundation, either version 3 of the License, or
 * (at your option) any later version.
 *
 * This program is distributed in the hope that it will be useful,
 * but WITHOUT ANY WARRANTY; without even the implied warranty of
 * MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
 * GNU General Public License for more details.
 *
 * You should have received a copy of the Lesser GNU General Public License
 * along with this program.  If not, see <http://www.gnu.org/licenses/>.
 * */
use dev_prefix::*;
use super::types::*;
use core;

pub fn get_subcommand<'a, 'b>() -> App<'a, 'b> {
    SubCommand::with_name("check")
        .about("check for any errors in the project")
        .settings(&[AS::DeriveDisplayOrder, COLOR])
}

// Helper functions
fn paint_it<W: Write>(w: &mut W, msg: &str) {
    if COLOR_IF_POSSIBLE {
        write!(w, "{}", Red.paint(msg)).unwrap();
    } else {
        write!(w, "{}", msg).unwrap();
    }
}
fn paint_it_bold<W: Write>(w: &mut W, msg: &str) {
    if COLOR_IF_POSSIBLE {
        write!(w, "{}", Red.bold().paint(msg)).unwrap();
    } else {
        write!(w, "{}", msg).unwrap();
    }
}

fn display_invalid_partof<W: Write>(w: &mut W, cwd: &Path, project: &Project) -> u64 {
    let mut error: u64 = 0;

    // display invalid partof names and locations
    let mut invalid_partof = ArtNames::new();

    // display artifacts with invalid partof names
    let mut displayed_header = false;
    for (name, artifact) in &project.artifacts {
        invalid_partof.clear();
        for p in &artifact.partof {
            if !project.artifacts.contains_key(p) {
                invalid_partof.insert(p.clone());
            }
        }
        if !invalid_partof.is_empty() {
            error += 1;
            let mut msg = String::new();
            if !displayed_header {
                displayed_header = true;
                paint_it_bold(w, "\nFound partof names that do not exist:\n");
            }
            write!(msg,
                   "    {} [{}]: {:?}\n",
                   name,
                   utils::relative_path(&artifact.path, cwd).display(),
                   invalid_partof)
                .unwrap();
            paint_it(w, &msg);
        }
    }

    error
}

fn display_unresolvable<W: Write>(w: &mut W, project: &Project) -> u64 {
    let mut error: u64 = 0;

    // display unresolvable partof names
    let unresolved: Vec<(ArtNameRc, &Artifact)> = Vec::from_iter(project.artifacts
        .iter()
        .filter(|a| a.1.completed < 0. || a.1.tested < 0.)
        .map(|n| (n.0.clone(), n.1)));
    let unknown_names: HashSet<ArtNameRc> = HashSet::from_iter(unresolved.iter()
        .map(|u| u.0.clone()));

    if !unresolved.is_empty() {
        error += 1;
        let mut unresolved_partof: HashMap<ArtNameRc, HashSet<ArtNameRc>> = HashMap::new();
        for &(ref name, artifact) in &unresolved {
            let partof: HashSet<_> = artifact.partof
                .iter()
                .filter(|n| {
                    !project.artifacts.contains_key(n.as_ref()) ||
                    unknown_names.contains(n.as_ref())
                })
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
            for (name, partof) in &mut unresolved_partof {
                // find names in partof that have no unresolved partofs and remove them
                remove_names.clear();
                for p in partof.iter() {
                    if resolved_names.contains(p) {
                        remove_names.push(p.clone());
                    }
                }
                if !remove_names.is_empty() {
                    did_something = true;
                }
                for p in &remove_names {
                    partof.remove(p);
                }

                // if this artifact has no unresolved partofs, then it is considered resolved
                if partof.is_empty() {
                    resolved_names.insert(name.clone());
                    just_resolved.push(name.clone());
                }
            }
            if !just_resolved.is_empty() {
                did_something = true;
            }
            for r in &just_resolved {
                unresolved_partof.remove(r);
            }
            if !did_something {
                break;
            }
        }
        paint_it_bold(w,
                      "\nArtifacts partof contains at least one recursive reference:\n");
        let mut unresolved_partof: Vec<_> = unresolved_partof.drain()
            .map(|mut v| (v.0, v.1.drain().collect::<Vec<_>>()))
            .collect();
        unresolved_partof.sort_by(|a, b| a.0.cmp(&b.0));
        for (name, partof) in unresolved_partof.drain(0..) {
            let mut msg = String::new();
            write!(msg, "    {:<30}: {:?}\n", name.to_string(), partof).unwrap();
            write!(w, "{}", msg).unwrap();
        }
    }

    error
}

fn display_invalid_locs<W: Write>(w: &mut W, cwd: &Path, project: &Project) -> u64 {
    let mut error: u64 = 0;

    // display invalid locations
    if !project.dne_locs.is_empty() {
        error += 1;
        // reorganize them by file
        let mut invalid_locs: HashMap<PathBuf, Vec<(ArtName, Loc)>> = HashMap::new();
        for (name, loc) in &project.dne_locs {
            if !invalid_locs.contains_key(&loc.path) {
                invalid_locs.insert(loc.path.clone(), Vec::new());
            }
            invalid_locs.get_mut(&loc.path).unwrap().push((name.clone(), loc.clone()));
        }
        let header = "\nFound implementation links in the code that do not exist:\n";
        paint_it_bold(w, header);
        let mut invalid_locs: Vec<(PathBuf, Vec<(ArtName, Loc)>)> =
            Vec::from_iter(invalid_locs.drain());
        invalid_locs.sort_by(|a, b| a.0.cmp(&b.0));
        for (path, mut locs) in invalid_locs.drain(0..) {
            // sort by where they appear in the file
            let mut pathstr = String::new();
            write!(pathstr,
                   "    {}:\n",
                   utils::relative_path(&path, cwd).display())
                .unwrap();
            paint_it(w, &pathstr);
            locs.sort_by(|a, b| a.1.line_col.cmp(&b.1.line_col));
            for (name, loc) in locs {
                let mut loc_str = String::new();
                write!(loc_str, "    - ({}:{})", loc.line_col.0, loc.line_col.1).unwrap();
                paint_it(w, &loc_str);
                write!(w, " {}\n", name).unwrap();
            }
        }
    }

    error
}


fn display_hanging_artifacts<W: Write>(w: &mut W, cwd: &Path, project: &Project) -> u64 {
    let mut error: u64 = 0;

    // find hanging artifacts
    fn partof_types(a: &Artifact, types: &HashSet<ArtType>) -> bool {
        for p in &a.partof {
            if types.contains(&p.ty) {
                return true;
            }
        }
        false
    }
    let rsk_spc_types = HashSet::from_iter(vec![ArtType::RSK, ArtType::SPC]);
    let req_types = HashSet::from_iter(vec![ArtType::REQ]);

    let mut hanging: Vec<(ArtNameRc, &Path)> = Vec::new();
    for (name, artifact) in &project.artifacts {
        let ty = name.ty;
        if (ty != ArtType::REQ) && !artifact.is_parent() && !name.is_root() &&
           name.parent().unwrap().is_root() &&
           match ty {
            ArtType::TST => !partof_types(artifact, &rsk_spc_types),
            ArtType::SPC | ArtType::RSK => !partof_types(artifact, &req_types),
            _ => unreachable!(),
        } {
            hanging.push((name.clone(), &artifact.path));
        }
    }
    hanging.sort_by(|a, b| a.1.cmp(b.1));
    if !hanging.is_empty() {
        error += 1;
        let msg = "\nHanging artifacts found (top-level but not partof a higher type):\n";
        paint_it_bold(w, msg);
        for (h, p) in hanging {
            let mut msg = String::new();
            write!(msg,
                   "    {:<30}: {}\n",
                   utils::relative_path(p, cwd).display(),
                   h)
                .unwrap();
            write!(w, "{}", msg).unwrap();
        }
    }

    error
}

fn display_hanging_references<W: Write>(w: &mut W, cwd: &Path, project: &Project) -> u64 {
    let mut error: u64 = 0;

    let regexp = Regex::new(&format!(r"(?i)\[\[({})\]\]", core::types::ART_VALID_STR))
        .expect("tested regexp");
    let mut hanging: HashMap<ArtNameRc, Vec<ArtName>> = HashMap::new();

    for (name, artifact) in &project.artifacts {
        let mut found = vec![];
        for cap in regexp.captures_iter(&artifact.text) {
            let raw = cap.get(1).expect("regexp definition");
            let tname = ArtName::from_str(raw.as_str()).expect("regexp validatd");
            if !project.artifacts.contains_key(&tname) {
                error += 1;
                found.push(tname);
            }
        }
        if !found.is_empty() {
            hanging.insert(name.clone(), found);
        }
    }

    if !hanging.is_empty() {
        paint_it_bold(w,
                      "\nArtifacts text contains invalid [[ART-name]] references:\n");
        let mut hanging: Vec<_> = hanging.drain().collect();
        hanging.sort();
        for &(ref name, ref found) in &hanging {
            let artifact = project.artifacts.get(name).expect("inserted from");
            paint_it(w,
                     &format!("    {} ({}):\n",
                              name,
                              utils::relative_path(&artifact.path, cwd).display()));
            for f in found {
                write!(w, "    - {}", f).unwrap();
            }
        }
    }
    error
}

/// #SPC-cmd-check
#[allow(cyclomatic_complexity)] // TODO: break this up
pub fn run_cmd<W: Write>(w: &mut W, cwd: &Path, project: &Project) -> Result<()> {
    let mut error: u64 = 0;

    error += display_invalid_partof(w, cwd, project);
    error += display_unresolvable(w, project);
    error += display_invalid_locs(w, cwd, project);
    error += display_hanging_artifacts(w, cwd, project);
    error += display_hanging_references(w, cwd, project);

    if error == 0 {
        let mut msg = String::new();
        write!(msg, "art check: no errors found in {}\n", cwd.display()).unwrap();
        if COLOR_IF_POSSIBLE {
            write!(w, "{}", Green.paint(msg)).unwrap();
        } else {
            write!(w, "{}", msg).unwrap();
        }
    } else {
        write!(w, "\n").unwrap();
    }
    if error != 0 {
        Err(ErrorKind::CmdError("errors found during ls, see logs".to_string()).into())
    } else {
        Ok(())
    }
}
