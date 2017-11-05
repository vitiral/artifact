//! Methods for exporting artifact to other data types (like json)

use serde_json;
use uuid::Uuid;

use dev_prefix::*;
use types::*;
use cmd::check;
use utils::UUID;

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct FullLocsData {
    root: Option<LocData>,
    sublocs: HashMap<String, LocData>,
}

#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct LocData {
    pub path: String,
    pub line: u64,
}

#[derive(Serialize, Deserialize, Debug, Default, Clone, PartialEq)]
pub struct ArtifactData {
    pub id: u64,
    pub revision: u64,
    pub name: String,
    pub def: String,
    pub text: String,
    pub partof: Vec<String>,

    #[serde(default)] pub subnames: Vec<String>,
    #[serde(default)] pub parts: Vec<String>,
    #[serde(default)] pub code: Option<FullLocsData>,
    #[serde(default)] pub done: Option<String>,
    #[serde(default = "default_comp_tested")] pub completed: f32,
    #[serde(default = "default_comp_tested")] pub tested: f32,
}

#[derive(Serialize, Debug, Default, Clone, PartialEq)]
pub struct ProjectData {
    pub artifacts: Vec<ArtifactData>,
    pub files: Vec<String>,
    pub checked: String,
    pub uuid: Uuid,
}

fn default_comp_tested() -> f32 {
    -1.0_f32
}

impl Project {
    pub fn to_data(&self) -> ProjectData {
        let artifacts = self.artifacts
            .iter()
            .map(|(n, a)| a.to_data(&self.origin, n))
            .collect();

        let files: Vec<String> = self.files
            .iter()
            .map(|p| {
                p.strip_prefix(&self.origin)
                    .expect("origin invalid")
                    .to_string_lossy()
                    .to_string()
            })
            .collect();

        let mut checked: Vec<u8> = Vec::new();
        let cmd = check::Cmd { color: false };

        check::display_check(&mut checked, &self.origin, self, &cmd);

        ProjectData {
            artifacts: artifacts,
            files: files,
            checked: String::from_utf8(checked).expect("invalid-utf8 from checked"),
            uuid: *UUID,
        }
    }
}

impl Artifact {
    /// convert an `Artifact` to it's data form
    pub fn to_data(&self, origin: &Path, name: &NameRc) -> ArtifactData {
        let (code, done) = match self.done {
            Done::Code(ref l) => {
                let root = match l.root {
                    Some(ref r) => Some(LocData {
                        path: r.path
                            .strip_prefix(origin)
                            .expect("origin invalid")
                            .to_string_lossy()
                            .to_string(),
                        line: r.line as u64,
                    }),
                    None => None,
                };
                let sublocs = HashMap::from_iter(l.sublocs.iter().map(|(n, l)| {
                    let loc = LocData {
                        path: l.path
                            .strip_prefix(origin)
                            .expect("origin invalid")
                            .to_string_lossy()
                            .to_string(),
                        line: l.line as u64,
                    };
                    (n.to_string(), loc)
                }));
                let full_loc = FullLocsData {
                    root: root,
                    sublocs: sublocs,
                };
                (Some(full_loc), None)
            }
            Done::Defined(ref s) => (None, Some(s.clone())),
            Done::NotDone => (None, None),
        };
        let mut partof: Vec<_> = self.partof.iter().map(|n| n.raw.clone()).collect();
        let mut parts: Vec<_> = self.parts.iter().map(|n| n.raw.clone()).collect();

        partof.sort();
        parts.sort();
        let path = self.def
            .strip_prefix(origin)
            .expect("origin invalid")
            .to_string_lossy()
            .to_string();
        let mut subnames: Vec<_> = self.subnames.iter().map(|n| n.to_string()).collect();
        subnames.sort();

        ArtifactData {
            id: self.id,
            revision: self.revision,
            name: name.raw.clone(),
            def: path,
            text: self.text.clone(),
            subnames: subnames,
            partof: partof,
            parts: parts,
            code: code,
            done: done,
            completed: self.completed,
            tested: self.tested,
        }
    }

    /// Get an `Artifact` from it's data form
    pub fn from_data(repo: &Path, data: &ArtifactData) -> Result<(NameRc, Artifact)> {
        let name = try!(NameRc::from_str(&data.name));
        let mut partof: HashSet<NameRc> = HashSet::new();
        for p in &data.partof {
            let pname = try!(NameRc::from_str(p));
            partof.insert(pname);
        }
        let done = if data.done.is_some() && data.code.is_some() {
            let msg = "has both done and code defined".to_string();
            return Err(ErrorKind::InvalidArtifact(data.name.clone(), msg).into());
        } else if let Some(ref d) = data.done {
            if d == "" {
                return Err(
                    ErrorKind::InvalidAttr(
                        name.to_string(),
                        "done cannot be an empty string.".to_string(),
                    ).into(),
                );
            }
            Done::Defined(d.clone())
        } else if let Some(ref c) = data.code {
            let root = match c.root {
                Some(ref r) => Some(Loc {
                    path: repo.join(&r.path),
                    line: r.line as usize,
                }),
                None => None,
            };
            let mut sublocs = HashMap::new();
            for (n, l) in &c.sublocs {
                let subname = SubName::from_str(n)?;
                let loc = Loc {
                    path: repo.join(&l.path),
                    line: l.line as usize,
                };
                sublocs.insert(subname, loc);
            }
            Done::Code(FullLocs {
                root: root,
                sublocs: sublocs,
            })
        } else {
            Done::NotDone
        };

        Ok((
            name,
            Artifact {
                id: data.id,
                revision: data.revision,
                def: repo.join(&data.def),
                text: data.text.clone(),
                subnames: HashSet::new(),
                partof: partof,
                done: done,
                parts: HashSet::new(),
                completed: -1.0,
                tested: -1.0,
            },
        ))
    }
}

/// convert the project's artifacts to a json list
pub fn project_artifacts_to_json(project: &Project, names: Option<&[NameRc]>) -> String {
    let out_arts: Vec<_> = if let Some(names) = names {
        names
            .iter()
            .map(|n| project.artifacts[n].to_data(&project.origin, n))
            .collect()
    } else {
        project
            .artifacts
            .iter()
            .map(|(n, a)| a.to_data(&project.origin, n))
            .collect()
    };

    let value = serde_json::to_value(out_arts).unwrap();
    serde_json::to_string(&value).unwrap()
}

#[test]
fn test_serde() {
    let artifact = ArtifactData {
        id: 10,
        revision: 0,
        name: "name".to_string(),
        def: "path".to_string(),
        text: "text".to_string(),
        subnames: Vec::new(),
        partof: Vec::from_iter(vec!["partof-1".to_string()]),
        parts: Vec::from_iter(vec!["part-1".to_string()]),
        done: None,
        code: Some(FullLocsData {
            root: Some(LocData {
                path: "path".to_string(),
                line: 10,
            }),
            sublocs: HashMap::new(),
        }),
        completed: 0.,
        tested: 0.,
    };

    let serialized = serde_json::to_string(&artifact).unwrap();
    let deserialized: ArtifactData = serde_json::from_str(&serialized).unwrap();

    assert_eq!(artifact, deserialized);


    // TODO: enable this test
    // load an artifact with defaults
    //    let with_defaults = r#"
    // {
    //    "id": 10,
    //    "name": "name",
    //    "path": "path",
    //    "text": "text",
    //    "partof": ["partof-1"],
    // }"#;
    //    let deserialized: ArtifactData = serde_json::from_str(with_defaults).unwrap();
    //    artifact.parts = vec![];
    //    artifact.loc = None;
    //    artifact.completed = -1;
    //    artifact.tested = -1;
    //    assert_eq!(artifact, deserialized);
}
