//! module that discovers artifact's links

use std::path;
use std::collections::{HashMap, HashSet};

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
        let parent = match name.parent() {
            Some(p) => p,
            None => continue,
        };
        artifact.partof.insert(parent);
    }
}

/// traverse all artifacts and their `partof` members and cross-link them to
/// the artifact's `parts` members
pub fn link_parts(artifacts: &mut Artifacts) {
    // get all the parts, linked by name
    let mut artifact_parts: HashMap<ArtName, HashSet<ArtName>> = HashMap::new();
    for (name, artifact) in artifacts.iter() {
        // get the artifacts this is a `partof`, this artifact should be in all of their `parts`
        for partof in artifact.partof.iter() {
            if !artifacts.contains_key(&partof) {
                println!("WARN: [{:?}] {} has invalid partof={}", artifact.path, name, partof);
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
}
