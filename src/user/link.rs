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

#[derive(Debug, Clone, Copy)]
/// define a part struct for keeping tally
struct Part {
    affects_completed: bool,
    affects_tested: bool,

    completed: f32,
    tested: f32,
}

/// Calculate the average of the artifact's 'parts'
fn parts_average(ty: Type, parts: &[&Part]) -> Part {
    let mut num_completed = 0;
    let mut sum_completed = 0.0;
    let mut num_tested = 0;
    let mut sum_tested = 0.0;

    match ty {
        Type::REQ => {
            // It is just the sum of it's parts no matter what
            for p in parts.iter() {
                num_completed += 1;
                sum_completed += p.completed;
                num_tested += 1;
                sum_tested += p.tested;
            }
        }
        _ => for p in parts.iter() {
            if p.affects_completed {
                num_completed += 1;
                sum_completed += p.completed;
            }
            if p.affects_tested {
                num_tested += 1;
                sum_tested += p.tested;
            }
        },
    }

    let (aff_spc, aff_tst) = match ty {
        Type::REQ => (true, true),
        Type::SPC => (true, false),
        Type::TST => (false, true),
    };

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

    Part {
        completed: completed,
        tested: tested,
        affects_completed: aff_spc,
        affects_tested: aff_tst,
    }
}

/// Get the calculated value of the artifact based on its `done` field
fn calc_done_field(ty: Type, artifact: &Artifact) -> Option<Part> {
    let (is_done, ratio) = match artifact.done {
        Done::Code(ref locs) => {
            if let Type::REQ = ty {
                panic!("REQ cannot have code links.");
            }
            (false, locs.ratio_complete(artifact.subnames.len()))
        }
        Done::Defined(_) => (true, 1.0),
        Done::NotDone => {
            if !artifact.parts.is_empty() {
                // @completion.link_nouse
                return None;
            }
            (false, 0.0)
        }
    };

    let (aff_comp, aff_tst) = if is_done {
        // #SPC-data-completion.link_done
        (true, true)
    } else {
        match ty {
            Type::REQ => (true, true),
            Type::SPC => (true, false),
            Type::TST => (false, true),
        }
    };

    Some(Part {
        completed: ratio,
        tested: ratio,
        affects_completed: aff_comp,
        affects_tested: aff_tst,
    })
}

/// @..final
fn part_final(ty: Type, part: &mut Part) {
    // All artifacts affect tested
    part.affects_tested = true;

    if let Type::TST = ty {
        part.completed = part.tested;
    }
}

/// Discover how complete and how tested all artifacts are (or are not!)
///
/// TODO: @SPC-data-completion
pub fn set_completed(artifacts: &mut Artifacts) -> usize {
    let mut names = Names::from_iter(artifacts.keys().cloned());
    let mut known: HashMap<NameRc, Part> = HashMap::with_capacity(names.len());
    let mut found = Names::with_capacity(names.len());

    while !names.is_empty() {
        for name in &names {
            let artifact = artifacts.get(name).unwrap();
            if !artifact.parts.iter().all(|n| known.contains_key(n)) {
                // not all children are yet known
                continue;
            }
            let done_part = calc_done_field(name.ty, artifact);

            let mut part = {
                let mut parts: Vec<_> = artifact
                    .parts
                    .iter()
                    .map(|n| known.get(n).expect("previously validated"))
                    .collect();

                if let Some(ref d) = done_part {
                    parts.push(d);
                }

                parts_average(name.ty, &parts)
            };

            part_final(name.ty, &mut part);
            found.insert(name.clone());
            known.insert(name.clone(), part);
        }
        if found.is_empty() {
            // No progress has been made, so we are done
            break;
        }
        for name in found.drain() {
            names.remove(&name);
        }
    }

    for (name, artifact) in artifacts.iter_mut() {
        // note: if it is not known if some were uncalculatable
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
