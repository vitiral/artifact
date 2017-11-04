/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! module that discovers artifact's links

use dev_prefix::*;
use types::*;

pub fn do_links(artifacts: &mut Artifacts) -> Result<()> {
    validate_done(artifacts)?;

    link_named_partofs(artifacts);
    link_parents(artifacts)?;

    validate_partof(artifacts)?;

    link_parts(artifacts);
    set_completed(artifacts);
    Ok(())
}

/// traverse all artifacts and link them to their by-name parent
pub fn link_parents(artifacts: &mut Artifacts) -> Result<()> {
    let names = Names::from_iter(artifacts.keys().cloned());
    for (name, artifact) in artifacts.iter_mut() {
        let parent = match name.parent_rc() {
            Some(p) => p,
            None => continue,
        };
        if !names.contains(&parent) {
            return Err(
                ErrorKind::MissingParent(name.raw.clone(), parent.raw.clone()).into(),
            );
        }
        artifact.partof.insert(parent);
    }
    Ok(())
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
            Done::NotDone => {}
            // correct!
            Done::Code(ref l) => {
                error!(
                    "{} was declared implemented in code at {}. {}",
                    name,
                    l,
                    valid_for
                );
                error = true;
            }
            Done::Defined(_) => {
                error!(
                    "{} was defined as done at {}. {}",
                    name,
                    artifact.def.display(),
                    valid_for
                );
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
                (&Type::SPC, &Type::SPC) |
                (&Type::SPC, &Type::REQ) |
                (&Type::TST, &Type::TST) |
                (&Type::TST, &Type::SPC) => {}
                (_, _) => {
                    error!(
                        "[{:?}:{}]: {:?} can not be a partof {:?}",
                        artifact.def,
                        name,
                        p_type,
                        n_type
                    );
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
                debug!(
                    "[{:?}] {} has invalid partof = {}",
                    artifact.def,
                    name,
                    partof
                );
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
    /// define a part struct for keeping tally
    #[derive(Debug, Clone, Copy)]
    struct Part {
        tested: f32,
        completed: f32,
        count_spc_completed: bool,
        count_spc_tested: bool,
    }

    let mut names = Names::from_iter(artifacts.keys().cloned());
    let mut known: HashMap<NameRc, Part> = HashMap::with_capacity(names.len());
    let mut found = Names::with_capacity(names.len());

    while !names.is_empty() {
        for name in &names {
            // if this name is calcu
            let artifact = artifacts.get(name).unwrap();
            if !artifact.parts.iter().all(|n| known.contains_key(n)) {
                continue;
            }

            // we know the artifact parts are are all completed, we just need
            // to calculate
            let mut parts: Vec<Part> = Vec::from_iter(
                artifact
                    .parts
                    .iter()
                    .map(|n| known.get(n).expect("previously validated"))
                    .cloned(),
            );

            // Push the "done" field
            match (&artifact.done, name.ty) {
                (&Done::Code(_), Type::TST) => {
                    // it is a completed test, but it does not count towards
                    // "completed" for spcs
                    // ... since we are currently processing a TST this information
                    // might as well be useless though...
                    parts.push(Part {
                        tested: 1.0,
                        completed: 1.0,
                        count_spc_completed: false,
                        count_spc_tested: true,
                    });
                }
                (&Done::Code(_), Type::SPC) => {
                    // it is a completed spec, but it does not count towards "tested"
                    parts.push(Part {
                        tested: 1.0,
                        completed: 1.0,
                        count_spc_completed: true,
                        count_spc_tested: false,
                    });
                }
                (&Done::Code(_), Type::REQ) => unreachable!("validation prevents"),
                (&Done::Defined(_), _) => {
                    // `done` field always counts for both tested and completed
                    parts.push(Part {
                        tested: 1.0,
                        completed: 1.0,
                        count_spc_completed: true,
                        count_spc_tested: true,
                    });
                }
                (&Done::NotDone, _) => {}
            };

            let mut num_completed = 0;
            let mut sum_completed = 0.0;
            let mut num_tested = 0;
            let mut sum_tested = 0.0;

            match name.ty {
                // special calculation for the SPC type
                Type::SPC => for p in &parts {
                    if p.count_spc_completed {
                        num_completed += 1;
                        sum_completed += p.completed;
                    }
                    if p.count_spc_tested {
                        num_tested += 1;
                        sum_tested += p.tested;
                    }
                },
                _ => for p in &parts {
                    num_completed += 1;
                    sum_completed += p.completed;
                    num_tested += 1;
                    sum_tested += p.tested;
                },
            }
            let completed = if num_completed == 0 {
                0.0
            } else {
                sum_completed / num_completed as f32
            };

            let tested = if num_tested == 0 {
                0.0
            } else {
                sum_tested / num_tested as f32
            };

            // TST never counts towards SPC completion
            let count_spc_completed = match name.ty {
                Type::TST => false,
                _ => true,
            };

            let part = Part {
                completed: completed,
                tested: tested,
                count_spc_tested: true,
                count_spc_completed: count_spc_completed,
            };

            found.insert(name.clone());
            known.insert(name.clone(), part);
        }
        if found.is_empty() {
            break;
        }
        for name in found.drain() {
            names.remove(&name);
        }
    }
    for (name, artifact) in artifacts.iter_mut() {
        // note: if it is not known then it was un-calcuable
        if let Some(p) = known.get(name) {
            artifact.completed = p.completed;
            artifact.tested = p.tested;
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
        let mut artifacts = test_data::load_toml_simple(
            "\
            [REQ-one]
            [SPC-one]
            [TST-one]\n",
        );
        let req_one = NameRc::from_str("REQ-one").unwrap();
        let spc_one = NameRc::from_str("SPC-one").unwrap();
        let tst_one = NameRc::from_str("TST-one").unwrap();
        link_named_partofs(&mut artifacts);
        assert_eq!(artifacts.get(&req_one).unwrap().partof, Names::new());
        assert_eq!(
            artifacts.get(&spc_one).unwrap().partof,
            Names::from_iter(vec![req_one.clone()])
        );
        assert_eq!(
            artifacts.get(&tst_one).unwrap().partof,
            Names::from_iter(vec![spc_one.clone()])
        );
    }

    #[test]
    fn test_done() {
        let req_foo = NameRc::from_str("REQ-foo").unwrap();
        let spc_foo = NameRc::from_str("spc-foo").unwrap();
        let spc_bar = NameRc::from_str("spc-bar").unwrap();

        let mut artifacts = test_data::load_toml_simple(test_data::TOML_DONE);
        assert_eq!(
            artifacts.get(&spc_foo).unwrap().done,
            Done::Defined("foo".to_string())
        );

        do_links(&mut artifacts).unwrap();
        assert_eq!(artifacts.get(&req_foo).unwrap().completed, 1.0);
        assert_eq!(artifacts.get(&req_foo).unwrap().tested, 1.0);

        assert_eq!(artifacts.get(&spc_bar).unwrap().completed, 1.0);
        assert_eq!(artifacts.get(&spc_bar).unwrap().tested, 1.0);
    }

    #[test]
    fn test_invalid_partof() {
        use test_data::load_toml_simple;
        use user::link::validate_partof;

        let artifacts = load_toml_simple("[REQ-foo]\npartof = 'SPC-bar'\n");
        assert!(validate_partof(&artifacts).is_err());
        let artifacts = load_toml_simple("[REQ-foo]\npartof = 'TST-bar'\n");
        assert!(validate_partof(&artifacts).is_err());

        let artifacts = load_toml_simple("[SPC-foo]\npartof = 'TST-bar'\n");
        assert!(validate_partof(&artifacts).is_err());
        let artifacts = load_toml_simple("[TST-foo]\npartof = 'REQ-bar'\n");
        assert!(validate_partof(&artifacts).is_err());
    }
}
