/*  artifact: the requirements tracking tool made for developers
 * Copyright (C) 2018 Rett Berg <@vitiral, vitiral@gmail.com>
 *
 * The source code is Licensed under either of
 *
 * * Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or
 *   http://www.apache.org/licenses/LICENSE-2.0)
 * * MIT license ([LICENSE-MIT](LICENSE-MIT) or
 *   http://opensource.org/licenses/MIT)
 *
 * at your option.
 *
 * Unless you explicitly state otherwise, any contribution intentionally submitted
 * for inclusion in the work by you, as defined in the Apache-2.0 license, shall
 * be dual licensed as above, without any additional terms or conditions.
 * */
//! Created "expected" types for assertions and test frameworks
use super::*;
use crate::dev_prelude::*;
use path_abs::ser::ToStfu8;

#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "op", rename_all = "lowercase")]
pub enum ArtifactOpAssert {
    Create {
        artifact: ArtifactImAssert,
    },
    Update {
        artifact: ArtifactImAssert,
        name: Name,
        /// Example: "gQ7cdQ7bvyIoaUTEUsxMsg"
        id: Option<HashIm>,
    },
    Delete {
        name: Name,
        /// Example: "gQ7cdQ7bvyIoaUTEUsxMsg"
        id: Option<HashIm>,
    },
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactImAssert {
    pub name: Name,
    pub file: String,
    pub partof: IndexSet<Name>,
    pub done: Option<String>,
    pub text: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectAssert {
    pub settings: SettingsAssert,
    pub code_impls: IndexMap<Name, ImplCodeAssert>,
    pub artifacts: IndexMap<Name, ArtifactAssert>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SettingsAssert {
    #[serde(default = "default_settings")]
    pub settings_path: String,
    pub code_paths: Vec<String>,
    pub exclude_code_paths: Vec<String>,
    pub artifact_paths: Vec<String>,
    pub exclude_artifact_paths: Vec<String>,
    pub code_url: Option<String>,

    #[serde(default)]
    pub parse: SettingsParse,
    #[serde(default)]
    pub format: SettingsFormat,
    #[serde(default)]
    pub export: SettingsExport,
}

fn default_settings() -> String {
    ".art/settings.toml".to_string()
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ArtifactAssert {
    pub name: Name,
    pub file: String,
    pub partof: IndexSet<Name>,
    pub parts: IndexSet<Name>,
    pub completed: Completed,
    pub text: String,
    pub impl_: ImplAssert,
    pub subnames: IndexSet<SubName>,
}

#[derive(Debug, Serialize, Deserialize)]
#[serde(untagged)]
pub enum ImplAssert {
    Done(String),
    Code(ImplCodeAssert),
    NotImpl,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ImplCodeAssert {
    primary: Option<CodeLocAssert>,
    secondary: IndexMap<String, CodeLocAssert>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct CodeLocAssert {
    pub file: String,
    pub line: u64,
}

#[derive(Debug, Serialize, Deserialize)]
/// Assertions for categorized lints.
pub struct CategorizedAssert {
    pub error: Vec<lint::Lint>,
    pub other: Vec<lint::Lint>,
}

impl ArtifactImAssert {
    pub fn expected(self, base: &PathDir) -> ArtifactIm {
        let mut out = ArtifactIm {
            name: self.name,
            file: expect!(base.concat(self.file)).into(),
            partof: self.partof,
            done: self.done,
            text: self.text,
        };
        out.clean();
        out
    }
}

impl ProjectAssert {
    /// Get the "expected" value based on this assertion object.
    pub fn expected(mut self, base: &PathDir) -> Project {
        let mut out = Project {
            settings: self.settings.expected(base),
            code_impls: self
                .code_impls
                .drain(..)
                .map(|(name, impl_)| (name, impl_.expected(base.as_ref())))
                .collect(),
            artifacts: self
                .artifacts
                .drain(..)
                .map(|(name, art)| (name, art.expected(base.as_ref())))
                .collect(),
        };
        out.sort();
        out
    }
}

impl SettingsAssert {
    pub fn expected(self, base: &PathDir) -> Settings {
        let settings_path = PathAbs::new_unchecked(expect!(base.concat(self.settings_path)));
        Settings {
            base: base.clone(),
            settings_path: PathFile::new_unchecked(settings_path),
            code_paths: prefix_paths(base.as_ref(), &self.code_paths),
            exclude_code_paths: prefix_paths(base.as_ref(), &self.exclude_code_paths),
            artifact_paths: prefix_paths(base.as_ref(), &self.artifact_paths),
            exclude_artifact_paths: prefix_paths(base.as_ref(), &self.exclude_artifact_paths),
            code_url: self.code_url,

            parse: self.parse,
            format: self.format,
            export: self.export,
        }
    }
}

impl ArtifactAssert {
    pub fn expected(self, base: &PathAbs) -> Artifact {
        let mut art = Artifact {
            id: HashIm([0; 16]),
            name: self.name,
            file: expect!(base.concat(&self.file)).into(),
            partof: self.partof,
            parts: self.parts,
            completed: self.completed,
            text: self.text,
            impl_: self.impl_.expected(base),
            subnames: self.subnames,
        };

        art.id = ArtifactIm::from(art.clone()).hash_im();
        art
    }
}

impl ImplAssert {
    pub fn expected(self, base: &PathAbs) -> Impl {
        match self {
            ImplAssert::Done(d) => Impl::Done(d),
            ImplAssert::Code(c) => Impl::Code(c.expected(base)),
            ImplAssert::NotImpl => Impl::NotImpl,
        }
    }
}

impl ImplCodeAssert {
    pub fn expected(mut self, base: &PathAbs) -> ImplCode {
        ImplCode {
            primary: self.primary.map(|c| c.expected(base)),
            secondary: self
                .secondary
                .drain(..)
                .map(|(s, c)| (subname!(s), c.expected(base)))
                .collect(),
        }
    }
}

impl CodeLocAssert {
    fn expected(self, base: &PathAbs) -> CodeLoc {
        CodeLoc {
            file: join_abs(base, self.file),
            line: self.line,
        }
    }
}

pub trait LintExp {
    /// just mutate the lint to be correct
    fn make_expected(&mut self, base: &PathAbs);
}

impl LintExp for lint::Lint {
    fn make_expected(&mut self, base: &PathAbs) {
        if let Some(ref mut p) = self.path {
            *p = expect!(base.concat(&p)).to_stfu8();
        }
    }
}

impl CategorizedAssert {
    pub fn expected(mut self, base: &PathAbs) -> Categorized {
        let convert_lints = |lints: &mut Vec<lint::Lint>| {
            lints
                .iter_mut()
                .map(|l| {
                    l.make_expected(base);
                })
                .count();
        };
        convert_lints(&mut self.error);
        convert_lints(&mut self.other);
        Categorized {
            error: self.error,
            other: self.other,
        }
    }
}

// HELPERS

/// Add the path prefix to a list of strings
fn prefix_paths(base: &PathAbs, ends: &[String]) -> IndexSet<PathAbs> {
    ends.iter()
        .map(|e| match PathAbs::new(expect!(base.concat(e))) {
            Ok(p) => p,
            Err(e) => panic!("{}", e),
        })
        .collect()
}
