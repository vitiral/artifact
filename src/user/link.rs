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
//! module that discovers artifact's links

use dev_prefix::*;
use types::*;

pub fn do_links(artifacts: &mut Artifacts) -> Result<()> {
    validate_done(artifacts)?;

    link_named_partofs(artifacts); // MUST come before parents are created
    create_parents(artifacts);
    link_parents(artifacts);

    validate_partof(artifacts)?;

    link_parts(artifacts);
    set_completed(artifacts);
    set_tested(artifacts);
    Ok(())
}

/// create parents for all artifacts that have no parents
pub fn create_parents(artifacts: &mut Artifacts) {
    let mut create_names: Names = HashSet::new();
    for name in artifacts.keys() {
        let mut name = name.clone();
        loop {
            name = match (&name).parent_rc() {
                None => break,
                Some(p) => p,
            };
            if artifacts.contains_key(&name) || create_names.contains(&name) {
                // parent already exists, someone else will make sub-parents
                // (or sub-parents are already made)
                break;
            } else {
                create_names.insert(name.clone());
            }
        }
    }

    for name in create_names.drain() {
        let art = Artifact {
            path: PARENT_PATH.clone(),
            text: "AUTO".to_string(),
            partof: HashSet::new(),
            parts: HashSet::new(),
            done: Done::NotDone,
            completed: -1.0,
            tested: -1.0,
        };
        artifacts.insert(name, art);
    }
}

/// traverse all artifacts and link them to their by-name parent
pub fn link_parents(artifacts: &mut Artifacts) {
    for (name, artifact) in artifacts.iter_mut() {
        let parent = match name.parent_rc() {
            Some(p) => p,
            None => continue,
        };
        artifact.partof.insert(parent);
    }
}

/// traverse all artifacts and link them to their by-name type
pub fn link_named_partofs(artifacts: &mut Artifacts) {
    let artifacts_keys = Names::from_iter(artifacts.keys().cloned());
    for (name, artifact) in artifacts.iter_mut() {
        for p in name.named_partofs() {
            if artifacts_keys.contains(&p) {
                artifact.partof.insert(Arc::new(p));
            }
        }
    }
}

/// validate that only correct artifact types are defined as done
pub fn validate_done(artifacts: &Artifacts) -> Result<()> {
    let mut error = false;
    let valid_for = "Only valid for SPC and TST";
    for (name, artifact) in artifacts.iter() {
        match name.ty {
            Type::SPC | Type::TST => continue,
            _ => {}
        }
        match artifact.done {
            Done::NotDone => {} // correct!
            Done::Code(ref l) => {
                error!("{} was declared implemented in code at {}. {}",
                       name,
                       l,
                       valid_for);
                error = true;
            }
            Done::Defined(_) => {
                error!("{} was defined as done at {}. {}",
                       name,
                       artifact.path.display(),
                       valid_for);
                error = true;
            }
        }
    }
    if error {
        return Err(ErrorKind::InvalidDone.into());
    }
    Ok(())
}

pub fn validate_partof(artifacts: &Artifacts) -> Result<()> {
    let mut error = false;
    for (name, artifact) in artifacts.iter() {
        for partof in &artifact.partof {
            let n_type = name.ty;
            let p_type = partof.ty;
            match (&n_type, &p_type) {
                (&Type::REQ, &Type::REQ) |
                (&Type::RSK, &Type::RSK) |
                (&Type::RSK, &Type::REQ) |
                (&Type::SPC, &Type::SPC) |
                (&Type::SPC, &Type::REQ) |
                (&Type::TST, &Type::TST) |
                (&Type::TST, &Type::RSK) |
                (&Type::TST, &Type::SPC) => {}
                (_, _) => {
                    error!("[{:?}:{}]: {:?} can not be a partof {:?}",
                           artifact.path,
                           name,
                           p_type,
                           n_type);
                    error = true;
                }
            }
        }
    }
    if error {
        return Err(ErrorKind::InvalidPartof.into());
    }
    Ok(())
}

/// traverse all artifacts and their `partof` members and cross-link them to
/// the artifact's `parts` members
pub fn link_parts(artifacts: &mut Artifacts) -> u64 {
    // get all the parts, linked by name
    let mut warnings: u64 = 0;
    let mut artifact_parts: HashMap<NameRc, Names> = HashMap::new();
    for (name, artifact) in artifacts.iter() {
        // get the artifacts this is a `partof`, this artifact should be in all of their `parts`
        for partof in &artifact.partof {
            if !artifacts.contains_key(partof) {
                debug!("[{:?}] {} has invalid partof = {}",
                       artifact.path,
                       name,
                       partof);
                warnings += 1;
                continue;
            }
            // TODO: there is no get_key(K).clone() yet, so we can't re-use Rc data here
            // https://github.com/rust-lang/rfcs/pull/1175
            if !artifact_parts.contains_key(partof) {
                artifact_parts.insert(partof.clone(), HashSet::new());
            }
            artifact_parts.get_mut(partof).unwrap().insert(name.clone());
        }
    }
    // insert the parts
    for (name, parts) in artifact_parts.drain() {
        // trace!("* {} has parts {:?}", name, parts);
        artifacts.get_mut(&name).unwrap().parts = parts;
    }
    warnings
}


/// discover how complete and how tested all artifacts are (or are not!)
pub fn set_completed(artifacts: &mut Artifacts) -> usize {
    let mut names = Names::from_iter(artifacts.keys().cloned());
    let mut known = Names::new();
    let mut found = Names::new();
    while !names.is_empty() {
        for name in &names {
            // 0 means didn't find anything, 1 means calculate, 2 means 100%, 3 means 0%
            let mut got_it = 0;
            // create scope to use artifacts and modify it later
            {
                let artifact = artifacts.get(name).unwrap();
                // SPC and TST artifacts are done if loc is set
                if artifact.done.is_done() {
                    got_it = 2;
                } else if artifact.parts.is_empty() {
                    got_it = 3; // no parts and not done == 0% complete
                } else if artifact.parts.iter().all(|n| known.contains(n)) {
                    got_it = 1;
                }
            }
            // resolve artifact completeness
            match got_it {
                3 => artifacts.get_mut(name).unwrap().completed = 0.0,
                2 => artifacts.get_mut(name).unwrap().completed = 1.0,
                1 => {
                    artifacts.get_mut(name).unwrap().completed = {
                        let artifact = artifacts.get(name).unwrap();
                        // get the completed values, ignoring TSTs that are part of SPCs
                        let completed: Vec<f32> = if name.ty == Type::SPC {
                            artifact.parts
                                .iter()
                                .filter(|n| n.ty != Type::TST)
                                .map(|n| artifacts.get(n).unwrap().completed)
                                .collect()
                        } else {
                            artifact.parts
                                .iter()
                                .map(|n| artifacts.get(n).unwrap().completed)
                                .collect()
                        };
                        // now completed is just the sum of it's valid parts
                        match completed.len() {
                            0 => 0.0,
                            _ => {
                                completed.iter().fold(0.0, |sum, x| sum + x) /
                                completed.len() as f32
                            }
                        }
                    }
                }
                0 => {}
                _ => unreachable!(),
            }
            if got_it != 0 {
                // trace!("resolved {} at {}", name, artifacts.get(name).unwrap().completed);
                found.insert(name.clone());
                known.insert(name.clone());
            }
        }
        if found.is_empty() {
            break;
        }
        for name in found.drain() {
            names.remove(&name);
        }
    }
    names.len()
}

/// Find the amount each artifact is tested
pub fn set_tested(artifacts: &mut Artifacts) -> usize {
    let mut names = Names::from_iter(artifacts.keys().cloned());
    let mut known = Names::new();
    let mut found = Names::new();

    // TST.tested === TST.completed by definition
    for (name, artifact) in artifacts.iter_mut() {
        if name.ty == Type::TST && artifact.completed >= 0.0 {
            artifact.tested = artifact.completed;
            names.remove(name);
            known.insert(name.clone());
        } else if artifact.parts.is_empty() {
            artifact.tested = 0.0;
            names.remove(name);
            known.insert(name.clone());
        }
    }

    // everythign else is just the sum of their parts
    while !names.is_empty() {
        for name in &names {
            let mut got_it = false;
            {
                let artifact = artifacts.get(name).unwrap();
                if artifact.parts.iter().all(|n| known.contains(n)) {
                    got_it = true;
                }
            }
            if got_it {
                artifacts.get_mut(name).unwrap().tested = {
                    let artifact = artifacts.get(name).unwrap();
                    artifact.parts
                        .iter()
                        .map(|n| artifacts.get(n).unwrap().tested)
                        .fold(0.0, |sum, x| sum + x) /
                    artifact.parts.len() as f32
                };
                found.insert(name.clone());
                known.insert(name.clone());
            }
        }
        if found.is_empty() {
            break;
        }
        for name in found.drain() {
            names.remove(&name);
        }
    }
    names.len()
}

#[cfg(test)]
mod tests {
    use dev_prefix::*;
    use super::*;
    use test_data;

    #[test]
    fn test_link_named_partofs() {
        let mut artifacts = test_data::load_toml_simple("\
            [REQ-one]
            [SPC-one]
            [TST-one]
            [RSK-one]\n");
        let req_one = NameRc::from_str("REQ-one").unwrap();
        let spc_one = NameRc::from_str("SPC-one").unwrap();
        let tst_one = NameRc::from_str("TST-one").unwrap();
        let rsk_one = NameRc::from_str("RSK-one").unwrap();
        link_named_partofs(&mut artifacts);
        assert_eq!(artifacts.get(&req_one).unwrap().partof, Names::new());
        assert_eq!(artifacts.get(&spc_one).unwrap().partof,
                   Names::from_iter(vec![req_one.clone()]));
        assert_eq!(artifacts.get(&tst_one).unwrap().partof,
                   Names::from_iter(vec![spc_one.clone()]));
        assert_eq!(artifacts.get(&rsk_one).unwrap().partof, Names::new());
    }

    #[test]
    fn test_done() {
        let req_foo = NameRc::from_str("REQ-foo").unwrap();
        let spc_foo = NameRc::from_str("spc-foo").unwrap();

        let mut artifacts = test_data::load_toml_simple(test_data::TOML_DONE);
        assert_eq!(artifacts.get(&spc_foo).unwrap().done,
                   Done::Defined("foo".to_string()));

        do_links(&mut artifacts).unwrap();
        assert_eq!(artifacts.get(&req_foo).unwrap().completed, 1.0);
        assert_eq!(artifacts.get(&req_foo).unwrap().tested, 1.0);
    }

    #[test]
    fn test_invalid_partof() {
        use test_data::load_toml_simple;
        use user::link::validate_partof;

        let artifacts = load_toml_simple("[REQ-foo]\npartof = 'SPC-bar'\n");
        assert!(validate_partof(&artifacts).is_err());
        let artifacts = load_toml_simple("[REQ-foo]\npartof = 'RSK-bar'\n");
        assert!(validate_partof(&artifacts).is_err());
        let artifacts = load_toml_simple("[REQ-foo]\npartof = 'TST-bar'\n");
        assert!(validate_partof(&artifacts).is_err());

        let artifacts = load_toml_simple("[RSK-foo]\npartof = 'TST-bar'\n");
        assert!(validate_partof(&artifacts).is_err());
        let artifacts = load_toml_simple("[RSK-foo]\npartof = 'SPC-bar'\n");
        assert!(validate_partof(&artifacts).is_err());

        let artifacts = load_toml_simple("[SPC-foo]\npartof = 'TST-bar'\n");
        assert!(validate_partof(&artifacts).is_err());
        let artifacts = load_toml_simple("[SPC-foo]\npartof = 'RSK-bar'\n");
        assert!(validate_partof(&artifacts).is_err());

        let artifacts = load_toml_simple("[TST-foo]\npartof = 'REQ-bar'\n");
        assert!(validate_partof(&artifacts).is_err());
    }
}
