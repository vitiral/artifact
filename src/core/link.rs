//! module that discovers artifact's links

use std::path;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use core::types::{LoadResult, LoadError, Artifacts, Artifact, ArtType, ArtName};
use core::fmt;

/// create parents for all artifacts that have no parents except for
// [SPC-core-artifact-attrs-parts-parents-create]
pub fn create_parents(artifacts: &mut Artifacts) {
    let mut create_names: HashSet<ArtName> = HashSet::new();
    for (name, art) in artifacts.iter() {
        let mut name = name.clone();
        loop {
            name = match (&name).parent() {
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
            ty: name.get_type(),
            path: path::PathBuf::from("PARENT"),
            text: "Auto-created parent artifact".to_string(),
            refs: vec![],
            partof: HashSet::new(),
            parts: HashSet::new(),
            loc: None,
            completed: -1.0,
            tested: -1.0,
        };
        artifacts.insert(name, art);
    }
}

/// traverse all artifacts and link them to their by-name parent
/// [SPC-core-artifact-attrs-parts-parents-link]
pub fn link_parents(artifacts: &mut Artifacts) {
    for (name, artifact) in artifacts.iter_mut() {
        let parent = match name.parent() {
            Some(p) => p,
            None => continue,
        };
        artifact.partof.insert(parent);
    }
}

/// traverse all artifacts and link them to their by-name uppers
pub fn link_named_partofs(artifacts: &mut Artifacts) {
    let artifacts_keys: HashSet<ArtName> = HashSet::from_iter(artifacts.keys().cloned());
    for (name, artifact) in artifacts.iter_mut() {
        for p in name.named_partofs() {
            if artifacts_keys.contains(&p) {
                artifact.partof.insert(p);
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use super::super::*;
    use core::load::load_toml_simple;
    use std::collections::HashSet;
    use std::iter::FromIterator;
    #[test]
    fn test_link_named_partofs() {
        let mut artifacts = load_toml_simple("\
            [REQ-one]
            [SPC-one]
            [TST-one]
            [RSK-one]\n");
        let req_one = ArtName::from_str("REQ-one").unwrap();
        let spc_one = ArtName::from_str("SPC-one").unwrap();
        let tst_one = ArtName::from_str("TST-one").unwrap();
        let rsk_one = ArtName::from_str("RSK-one").unwrap();
        link_named_partofs(&mut artifacts);
        assert_eq!(artifacts.get(&req_one).unwrap().partof, HashSet::new());
        assert_eq!(artifacts.get(&spc_one).unwrap().partof, HashSet::from_iter(
            vec![req_one.clone()]));
        assert_eq!(artifacts.get(&tst_one).unwrap().partof, HashSet::from_iter(
            vec![spc_one.clone()]));
        assert_eq!(artifacts.get(&rsk_one).unwrap().partof, HashSet::new());
    }
}

pub fn validate_partof(artifacts: &Artifacts) -> LoadResult<()> {
    let mut error = false;
    for (name, artifact) in artifacts.iter() {
        for partof in artifact.partof.iter() {
            let n_type = name.get_type();
            let p_type = partof.get_type();
            match (&n_type, &p_type) {
                (&ArtType::REQ, &ArtType::REQ) => {}
                (&ArtType::RSK, &ArtType::RSK) | (&ArtType::RSK, &ArtType::REQ) => {}
                (&ArtType::SPC, &ArtType::SPC) | (&ArtType::SPC, &ArtType::REQ) => {}
                (&ArtType::TST, &ArtType::TST) |
                (&ArtType::TST, &ArtType::RSK) |
                (&ArtType::TST, &ArtType::SPC) => {}
                (_, _) => {
                    // [SPC-core-artifact-attrs-partof-validate]
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
        return Err(LoadError::new("Some artifacts have invalid partof attributes".to_string()));
    }
    Ok(())
}

/// traverse all artifacts and their `partof` members and cross-link them to
/// the artifact's `parts` members
pub fn link_parts(artifacts: &mut Artifacts) -> u64 {
    // get all the parts, linked by name
    let mut warnings: u64 = 0;
    let mut artifact_parts: HashMap<ArtName, HashSet<ArtName>> = HashMap::new();
    for (name, artifact) in artifacts.iter() {
        // get the artifacts this is a `partof`, this artifact should be in all of their `parts`
        for partof in artifact.partof.iter() {
            if !artifacts.contains_key(&partof) {
                warn!("[{:?}] {} has invalid partof = {}",
                      artifact.path,
                      name,
                      partof);
                warnings += 1;
                continue;
            }
            if !artifact_parts.contains_key(&partof) {
                artifact_parts.insert(partof.clone(), HashSet::new());
            }
            artifact_parts.get_mut(&partof).unwrap().insert(name.clone());
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
/// [SPC-core-coverage-percent-done]
pub fn set_completed(artifacts: &mut Artifacts) -> usize {
    let mut names: HashSet<ArtName> = HashSet::from_iter(artifacts.keys().map(|n| n.clone()));
    let mut known: HashSet<ArtName> = HashSet::new();
    let mut found: HashSet<ArtName> = HashSet::new();
    while names.len() > 0 {
        for name in names.iter() {
            // 0 means didn't find anything, 1 means calculate, 2 means 100%, 3 means 0%
            let mut got_it = 0;
            {
                // scope to use artifacts and modify it later
                let artifact = artifacts.get(&name).unwrap();
                // SPC and TST artifacts are done if loc is set
                match (&artifact.loc, &artifact.ty) {
                    (&Some(ref l), &ArtType::SPC) | (&Some(ref l), &ArtType::TST) => {
                        let lvalid = l.valid();
                        if lvalid {
                            got_it = 2; // it is 100% completed by definition
                        } else if !lvalid {
                            warn!("[{:?}:{}] has non-existant loc of {}",
                                  artifact.path,
                                  name,
                                  l);
                            got_it = 3; // it is 0% completed by definition
                        }
                    }
                    // [SPC-core-artifacts-attrs-loc-invalid]
                    (&Some(_), ty @ _) => warn!("[{:?}:{}] has loc set but is of type {:?}",
                                                    artifact.path, name, ty),
                    _ => {}
                }
                if got_it == 0 && artifact.parts.len() == 0 {
                    got_it = 3; // no parts or invalid-loc == 0% complete
                } else if got_it == 0 && artifact.parts.iter().all(|n| known.contains(&n)) {
                    got_it = 1;
                }
            }
            // resolve artifact completeness
            match got_it {
                3 => artifacts.get_mut(&name).unwrap().completed = 0.0,
                2 => artifacts.get_mut(&name).unwrap().completed = 1.0,
                1 => {
                    artifacts.get_mut(&name).unwrap().completed = {
                        let artifact = artifacts.get(&name).unwrap();
                        // get the completed values, ignoring TSTs that are part of SPCs
                        let completed: Vec<f32> = if artifact.ty == ArtType::SPC {
                            artifact.parts
                                    .iter()
                                    .filter(|n| artifacts.get(n).unwrap().ty != ArtType::TST)
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
        if found.len() == 0 {
            break;
        }
        for name in found.drain() {
            names.remove(&name);
        }
    }
    if names.len() != 0 {
        let mut ordered = Vec::from_iter(names.iter());
        ordered.sort();

        // warn!("could not resolve tested % for: [{}]", ordered.iter().map(|n| n.raw.clone())
        //       .join(", "));
        warn!("could not resolve completed % for Artifacts:");
        for name in ordered {
            let artifact = artifacts.get(name).unwrap();
            let mut unknown: Vec<_> = artifact.parts
                                              .iter()
                                              .filter(|n| !known.contains(n))
                                              .collect();
            unknown.sort();
            warn!(" - {} could not resolve parts: {}",
                  name,
                  fmt::names(&unknown));
        }
    }
    names.len()
}

/// Find the amount each artifact is tested
/// [SPC-core-coverage-percent-tested]
pub fn set_tested(artifacts: &mut Artifacts) -> usize {
    let mut names: HashSet<ArtName> = HashSet::from_iter(artifacts.keys().map(|n| n.clone()));
    let mut known: HashSet<ArtName> = HashSet::new();
    let mut found: HashSet<ArtName> = HashSet::new();

    // TST.tested === TST.completed by definition
    for (name, artifact) in artifacts.iter_mut() {
        if artifact.ty == ArtType::TST && artifact.completed >= 0.0 {
            artifact.tested = artifact.completed;
            names.remove(&name);
            known.insert(name.clone());
        } else if artifact.parts.len() == 0 {
            artifact.tested = 0.0;
            names.remove(&name);
            known.insert(name.clone());
        }
    }

    // everythign else is just the sum of their parts
    while names.len() > 0 {
        for name in names.iter() {
            let mut got_it = false;
            {
                let artifact = artifacts.get(&name).unwrap();
                if artifact.parts.iter().all(|n| known.contains(&n)) {
                    got_it = true;
                }
            }
            if got_it {
                artifacts.get_mut(&name).unwrap().tested = {
                    let artifact = artifacts.get(&name).unwrap();
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
        if found.len() == 0 {
            break;
        }
        for name in found.drain() {
            names.remove(&name);
        }
    }
    if names.len() != 0 {
        let mut ordered = Vec::from_iter(names.iter());
        ordered.sort();

        // warn!("could not resolve tested % for: [{}]", ordered.iter().map(|n| n.raw.clone())
        //       .join(", "));
        warn!("could not resolve tested % for Artifacts:");
        for name in ordered {
            let artifact = artifacts.get(name).unwrap();
            let mut unknown: Vec<_> = artifact.parts
                                              .iter()
                                              .filter(|n| !known.contains(n))
                                              .collect();
            unknown.sort();
            warn!(" - {} could not resolve parts: {}",
                  name,
                  fmt::names(&unknown));
        }
    }
    names.len()
}
