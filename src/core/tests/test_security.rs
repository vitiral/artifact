
use dev_prefix::*;
use core::prefix::*;
use core;
use core::tests;

#[test]
/// make sure that artifacts which are loaded "out of bounds"
/// don't make it past the security checker
/// partof: #TST-security-gen
fn test_bounds_checker() {
    let design = tests::TINVALID_BOUNDS.join("repo").join("design");
    let repo = core::find_repo(&design).unwrap();
    let project = core::load_path(&repo).unwrap();
    let req_bounds = ArtNameRc::from_str("REQ-bounds").unwrap();
    assert!(project.artifacts.contains_key(&req_bounds));
    assert_eq!(project.artifacts[&req_bounds].path,
               tests::TINVALID_BOUNDS.join("out_bounds.toml"));
    assert!(core::security::validate(&repo, &project).is_err());
}
