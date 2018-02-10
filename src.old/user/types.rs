/* Copyright (c) 2017 Garrett Berg, vitiral@gmail.com
 *
 * Licensed under the Apache License, Version 2.0, <LICENSE-APACHE or
 * http://apache.org/licenses/LICENSE-2.0> or the MIT license <LICENSE-MIT or
 * http://opensource.org/licenses/MIT>, at your option. This file may not be
 * copied, modified, or distributed except according to those terms.
 */
//! user: loading and saving user data
//!
//! This module encapsulates the loading and saving of artifacts

/// User options for an `Artifact`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct UserArtifact {
    pub done: Option<String>,
    pub partof: Option<UserPartof>,
    pub text: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(untagged)]
pub enum UserPartof {
    Single(String),
    Multi(Vec<String>),
}

/// User options for Settings
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct RawSettings {
    pub artifact_paths: Option<Vec<String>>,
    pub exclude_artifact_paths: Option<Vec<String>>,
    pub code_paths: Option<Vec<String>>,
    pub exclude_code_paths: Option<Vec<String>>,
    pub file_type: Option<String>,
}

#[test]
fn test_toml_simple() {
    use toml;
    use std::collections::BTreeMap;

    let raw = r#"[bar]
partof = [
    'foo',
    'bar',
]

[foo]
partof = 'bar'
"#;
    let value: BTreeMap<String, UserArtifact> = toml::from_str(raw).unwrap();
    assert_eq!(raw, toml::to_string_pretty(&value).unwrap());
}
