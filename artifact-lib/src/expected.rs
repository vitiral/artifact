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
//! Created "expected" types for assertions and test frameworks
use dev_prelude::*;
use super::*;

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
    pub paths: ProjectPathsAssert,
    pub code_impls: IndexMap<Name, ImplCodeAssert>,
    pub artifacts: IndexMap<Name, ArtifactAssert>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ProjectPathsAssert {
    pub code_paths: Vec<String>,
    pub exclude_code_paths: Vec<String>,
    pub artifact_paths: Vec<String>,
    pub exclude_artifact_paths: Vec<String>,
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
            file: PathArc::new(base.join(self.file)),
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
            paths: self.paths.expected(base),
            code_impls: self.code_impls
                .drain(..)
                .map(|(name, impl_)| (name, impl_.expected(base)))
                .collect(),
            artifacts: self.artifacts
                .drain(..)
                .map(|(name, art)| (name, art.expected(base)))
                .collect(),
        };
        out.sort();
        out
    }
}

impl ProjectPathsAssert {
    pub fn expected(self, base: &PathDir) -> ProjectPaths {
        ProjectPaths {
            base: base.clone(),
            code_paths: prefix_paths(base, &self.code_paths),
            exclude_code_paths: prefix_paths(base, &self.exclude_code_paths),
            artifact_paths: prefix_paths(base, &self.artifact_paths),
            exclude_artifact_paths: prefix_paths(base, &self.exclude_artifact_paths),
        }
    }
}

impl ArtifactAssert {
    pub fn expected(self, base: &PathAbs) -> Artifact {
        let mut art = Artifact {
            id: HashIm([0; 16]),
            name: self.name,
            file: PathArc::new(base.join(&self.file)),
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
            secondary: self.secondary
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

impl lint::Lint {
    /// just mutate the lint to be correct
    pub fn make_expected(&mut self, base: &PathAbs) {
        if let Some(ref mut p) = self.path {
            *p = PathArc::new(base.join(&p));
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
        .map(|e| match PathAbs::new(base.join(e)) {
            Ok(p) => p,
            Err(e) => panic!("{}", e),
        })
        .collect()
}
