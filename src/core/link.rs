//! module that discovers artifact's links

use std::path;
use std::collections::{HashMap, HashSet};
use std::iter::FromIterator;

use core::types::{
    LoadResult, LoadError,
    Artifacts, Artifact, ArtType, ArtName};

/// create parents for all artifacts that have no parents except for
/// LOC artifacts
pub fn create_parents(artifacts: &mut Artifacts) {
    let mut create_names: HashSet<ArtName> = HashSet::new();
    for (name, art) in artifacts.iter() {
        if art.ty == ArtType::LOC {
            continue;
        }
        let mut name = name.clone();
        loop {
            name = match (&name).parent() {
                None => break,
                Some(p) => p,
            };
            if create_names.contains(&name) {
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
            refs: vec!(),
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
pub fn link_parents(artifacts: &mut Artifacts) {
    for (name, artifact) in artifacts.iter_mut() {
        if name.get_type() == ArtType::LOC {
            continue;
        }
        let parent = match name.parent() {
            Some(p) => p,
            None => continue,
        };
        artifact.partof.insert(parent);
    }
}

pub fn validate_partof(artifacts: &Artifacts) -> LoadResult<()> {
    let mut error = false;
    for (name, artifact) in artifacts.iter() {
        for partof in artifact.partof.iter() {
            let n_type = name.get_type();
            let p_type = partof.get_type();
            match (&n_type, &p_type) {
                (&ArtType::REQ, &ArtType::REQ) => {},
                (&ArtType::RSK, &ArtType::RSK) | (&ArtType::RSK, &ArtType::REQ) => {},
                (&ArtType::SPC, &ArtType::SPC) | (&ArtType::SPC, &ArtType::REQ) => {},
                (&ArtType::TST, &ArtType::TST) | (&ArtType::TST, &ArtType::RSK)
                    | (&ArtType::TST, &ArtType::SPC) => {},
                (_, _) => {
                    println!("ERROR: [{:?}:{}]: {:?} can not be a partof {:?}",
                                artifact.path, name, p_type, n_type);
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
    let mut artifact_remove_parts: HashMap<ArtName, HashSet<ArtName>> = HashMap::new();
    for (name, artifact) in artifacts.iter() {
        // get the artifacts this is a `partof`, this artifact should be in all of their `parts`
        for partof in artifact.partof.iter() {
            if !artifacts.contains_key(&partof) {
                println!("WARN: [{:?}] {} has invalid partof={}", artifact.path, name, partof);
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
        artifacts.get_mut(&name).unwrap().parts = parts;
    }
    warnings
}


/// discover how complete and how tested all artifacts are (or are not!)
pub fn set_completed(artifacts: &mut Artifacts) -> usize {
    let mut names: HashSet<ArtName>  = HashSet::from_iter(artifacts.keys().map(|n| n.clone()));
    let mut known: HashSet<ArtName>  = HashSet::new();
    let mut found: HashSet<ArtName> = HashSet::new();
    while names.len() > 0 {
        for name in names.iter() {
            // 0 means didn't find anything, 1 means calculate, 2 means 100%, 3 means 0%
            let mut got_it = 0;
            { // scope to use artifacts and modify it later
                let artifact = artifacts.get(&name).unwrap();
                // SPC and TST artifacts are done if loc is set
                match (&artifact.loc, &artifact.ty) {
                    (&Some(ref l), &ArtType::SPC) | (&Some(ref l), &ArtType::TST) => {
                        if artifacts.contains_key(&l.loc) || l.valid() {
                            got_it = 2; // it is 100% completed by definition
                        } else if !l.valid() {
                            println!("WARN: [{:?}:{}] has non-existant loc", artifact.path, name);
                            got_it = 3; // it is 0% completed by definition
                        }
                    }
                    _ => {},
                }
                if got_it == 0 && artifact.parts.len() == 0 {
                    got_it = 3; // no parts or invalid-loc == 0% complete
                }
                else if got_it == 0 && artifact.parts.iter().all(|n| known.contains(&n)) {
                    got_it = 1;
                }
            }
            // resolve artifact completeness
            match got_it {
                3 => artifacts.get_mut(&name).unwrap().completed = 0.0,
                2 => artifacts.get_mut(&name).unwrap().completed = 100.0,
                1 => artifacts.get_mut(&name).unwrap().completed = {
                    // equal to sum of it's parts
                    let artifact = artifacts.get(&name).unwrap();
                    let out = artifact.parts.iter()
                        .map(|n| artifacts.get(n).unwrap().completed)
                        .fold(0.0, |sum, x| sum + x) / artifact.parts.len() as f32;
                    out
                },
                0 => {},
                _ => unreachable!(),
            }
            if got_it != 0 {
                println!(" *** resolved {} at {}", name, artifacts.get(name).unwrap().completed);
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
        println!("WARN: could not resolve completed % for: {:?}", names);
    }
    names.len()
}

/// Find the amount each artifact is tested
pub fn set_tested(artifacts: &mut Artifacts) -> usize {
    let mut names: HashSet<ArtName>  = HashSet::from_iter(artifacts.keys().map(|n| n.clone()));
    let mut known: HashSet<ArtName>  = HashSet::new();
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
                    artifact.parts.iter()
                        .map(|n| artifacts.get(n).unwrap().tested)
                        .fold(0.0, |sum, x| sum + x) / artifact.parts.len() as f32
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
        println!("WARN: could not resolve tested % for: {:?}", names);
    }
    names.len()
}

