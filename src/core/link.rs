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
            let n_type = name.get_type();
            let p_type = partof.get_type();
            match (name.get_type(), partof.get_type()) {
                (ArtType::REQ, ArtType::REQ) => {},
                (ArtType::RSK, ArtType::RSK) | (ArtType::RSK, ArtType::REQ) => {},
                (ArtType::SPC, ArtType::SPC) | (ArtType::SPC, ArtType::REQ) => {},
                (ArtType::TST, ArtType::TST) | (ArtType::TST, ArtType::RSK)
                    | (ArtType::TST, ArtType::SPC) => {},
                (_, _) => {
                    println!("WARN: [{:?}:{}]: {:?} should not be a partof {:?}",
                             artifact.path, name, p_type, n_type);
                    warnings += 1;
                }
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
            let mut got_it = 0; // 0 means didn't find anything, 1 means calculate, 2 means 100%
            { // use artifacts
                let artifact = artifacts.get(&name).unwrap();
                // SPC and TST artifacts are done if loc is set
                match (&artifact.ty, &artifact.loc) {
                    (&ArtType::SPC, &Some(ref l)) | (&ArtType::TST, &Some(ref l)) => {
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
                    got_it = 3; // no parts or loc == 0% complete
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
                    let artifact = artifacts.get(&name).unwrap();
                    artifact.parts.iter()
                        .map(|n| artifacts.get(n).unwrap().completed)
                        .fold(0.0, |sum, x| sum + x) / artifact.parts.len() as f32
                },
                0 => {},
                _ => unreachable!(),
            }
            if got_it != 0 {
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
    println!("WARN: could not resolve completeness for: {:?}", names);
    names.len()
}
