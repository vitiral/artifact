use std::iter::FromIterator;

use serde_json;

use super::super::*;

#[test]
fn test_serde() {
    let artifact = ArtifactData {
        id: 10,
        name: "name".to_string(),
        text: "text".to_string(),
        partof: Vec::from_iter(vec!["partof-1".to_string()]),
        parts: Vec::from_iter(vec!["part-1".to_string()]),
        loc: LocData { path: "path".to_string(), row: 10, col: 10 },
        completed: 0.,
        tested: 0.,
    };

    let serialized = serde_json::to_string(&artifact).unwrap();
    println!("serialized = {}", serialized);
    let deserialized: ArtifactData = serde_json::from_str(&serialized).unwrap();

    assert_eq!(artifact, deserialized);
}

