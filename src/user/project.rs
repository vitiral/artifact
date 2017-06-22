//! methods and functions for operating on Project structs.

use dev_prefix::*;
use types::*;

// Public Traits

impl Project {
    /// better than equal... has reasons why NOT equal!
    pub fn equal(&self, other: &Project) -> Result<()> {
        names_equal(&self.artifacts, &other.artifacts)?;
        attr_equal(
            "path",
            &self.artifacts,
            &other.artifacts,
            &|a: &Artifact| &a.def,
        )?;
        attr_equal(
            "text",
            &self.artifacts,
            &other.artifacts,
            &|a: &Artifact| &a.text,
        )?;
        attr_equal(
            "partof",
            &self.artifacts,
            &other.artifacts,
            &|a: &Artifact| &a.partof,
        )?;
        attr_equal(
            "parts",
            &self.artifacts,
            &other.artifacts,
            &|a: &Artifact| &a.parts,
        )?;
        attr_equal(
            "done",
            &self.artifacts,
            &other.artifacts,
            &|a: &Artifact| &a.done,
        )?;
        float_equal(
            "completed",
            &self.artifacts,
            &other.artifacts,
            &|a: &Artifact| a.completed,
        )?;
        float_equal(
            "tested",
            &self.artifacts,
            &other.artifacts,
            &|a: &Artifact| a.tested,
        )?;
        proj_attr_equal("origin", &self.origin, &other.origin)?;
        proj_attr_equal("settings", &self.settings, &other.settings)?;
        proj_attr_equal("files", &self.files, &other.files)?;
        proj_attr_equal("dne_locs", &self.dne_locs, &other.dne_locs)?;
        proj_attr_equal("repo_map", &self.repo_map, &other.repo_map)?;
        Ok(())
    }
}

// Private Methods

/// assert that two artifact groups have the same name keys
fn names_equal(a: &Artifacts, b: &Artifacts) -> Result<()> {
    let a_keys: HashSet<NameRc> = a.keys().cloned().collect();
    let b_keys: HashSet<NameRc> = b.keys().cloned().collect();
    if b_keys != a_keys {
        let missing = a_keys.symmetric_difference(&b_keys).collect::<Vec<_>>();
        let msg = format!(
            "missing artifacts: {:?}\nFIRST:\n{:?}\nSECOND:\n{:?}",
            missing,
            a_keys,
            b_keys
        );
        Err(ErrorKind::NotEqual(msg).into())
    } else {
        Ok(())
    }
}

/// assert that the attributes are equal on the artifact
/// if they are not, then find what is different and include
/// in the error description.
///
/// This is very expensive for values that differ
fn attr_equal<T, F>(attr: &str, a: &Artifacts, b: &Artifacts, get_attr: &F) -> Result<()>
where
    T: Debug + PartialEq,
    F: Fn(&Artifact) -> &T,
{
    let mut diff: Vec<String> = Vec::new();

    for (a_name, a_art) in a.iter() {
        let b_art = b.get(a_name).unwrap();
        let a_attr = get_attr(a_art);
        let b_attr = get_attr(b_art);
        if a_attr != b_attr {
            let mut a_str = format!("{:?}", a_attr);
            let mut b_str = format!("{:?}", b_attr);
            let a_big = if a_str.len() > 100 { "..." } else { "" };
            let b_big = if b_str.len() > 100 { "..." } else { "" };
            a_str.truncate(100);
            b_str.truncate(100);
            diff.push(format!(
                "[{}: {}{} != {}{}]",
                a_name,
                a_str,
                a_big,
                b_str,
                b_big
            ));
        }
    }

    if diff.is_empty() {
        Ok(())
    } else {
        Err(
            ErrorKind::NotEqual(format!("{} different: {:?}", attr, diff)).into(),
        )
    }
}

/// num *approximately* equal
fn float_equal<F>(attr: &str, a: &Artifacts, b: &Artifacts, get_num: &F) -> Result<()>
where
    F: Fn(&Artifact) -> f32,
{
    let mut diff: Vec<String> = Vec::new();
    fn thous(f: f32) -> i64 {
        (f * 1000.) as i64
    }

    for (a_name, a_art) in a.iter() {
        let b_art = b.get(a_name).unwrap();
        let a_attr = get_num(a_art);
        let b_attr = get_num(b_art);
        if thous(a_attr) != thous(b_attr) {
            let mut a_str = format!("{:?}", a_attr);
            let mut b_str = format!("{:?}", b_attr);
            a_str.truncate(50);
            b_str.truncate(50);
            diff.push(format!("({}, {} != {})", a_name, a_str, b_str));
        }
    }

    if diff.is_empty() {
        Ok(())
    } else {
        Err(
            ErrorKind::NotEqual(format!("{} different: {:?}", attr, diff)).into(),
        )
    }
}

fn proj_attr_equal<T>(attr: &str, a: &T, b: &T) -> Result<()>
where
    T: Debug + PartialEq,
{
    if a != b {
        Err(
            ErrorKind::NotEqual(format!("{} FIRST:\n{:?}\n\nSECOND:\n{:?}", attr, a, b)).into(),
        )
    } else {
        Ok(())
    }
}
