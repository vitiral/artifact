
use dev_prefix::*;
use types::*;
use user;
use test_data;

use api::crud;
use api::utils;


#[test]
fn test_split() {
    let simple = &test_data::TSIMPLE_DIR;
    let design = simple.join("design");

    let p = user::load_repo(simple.as_path()).unwrap();
    let starting_len = p.artifacts.len();

    let data_artifacts: Vec<_> = p.artifacts
        .iter()
        .map(|(n, a)| a.to_data(&p.origin, n))
        .collect();

    let req_purpose = NameRc::from_str("REQ-purpose").unwrap();

    let mut changed_data = data_artifacts
        .iter()
        .filter(|a| a.name == "REQ-purpose")
        .next()
        .unwrap()
        .clone();

    // changing text is OK
    {
        changed_data.text = "changed to something else".to_string();
        let new_artifacts = vec![changed_data.clone()];

        let (unchanged_artifacts, save_artifacts) =
            utils::split_artifacts(&p, &data_artifacts, &new_artifacts, false).unwrap();

        assert_eq!(save_artifacts.len(), 1);
        let new_data = save_artifacts
            .get(&req_purpose)
            .unwrap()
            .to_data(&p.origin, &req_purpose);
        assert_eq!(new_data.text, changed_data.text);
        assert_eq!(unchanged_artifacts.len(), starting_len - 1);
    }

    // changing the path to a valid place is OK
    {
        let original = changed_data.def;

        let new_path = design.join("lvl_1/req.toml").to_string_lossy().to_string();
        changed_data.def = new_path.clone();

        let new_artifacts = vec![changed_data.clone()];
        let (_, save_artifacts) =
            utils::split_artifacts(&p, &data_artifacts, &new_artifacts, false).unwrap();
        let new_data = save_artifacts
            .get(&req_purpose)
            .unwrap()
            .to_data(&p.origin, &req_purpose);
        assert_eq!(
            Path::new(&new_data.def),
            Path::new(&new_path)
                .strip_prefix(&simple.as_path())
                .unwrap()
        );

        changed_data.def = original;
    }

    // changing path to new path NOT ok
    {
        let original = changed_data.def;
        changed_data.def = design.join("dne.toml").to_string_lossy().to_string();
        let new_artifacts = vec![changed_data.clone()];
        assert!(utils::split_artifacts(&p, &data_artifacts, &new_artifacts, false).is_err());
        changed_data.def = original;
    }

    // having unmatching id NOT ok
    {
        let original = changed_data.id;
        changed_data.id = 42424242;
        let new_artifacts = vec![changed_data.clone()];
        assert!(utils::split_artifacts(&p, &data_artifacts, &new_artifacts, false).is_err());
        changed_data.id = original;
    }

    // having invalid name NOT ok
    {
        let original = changed_data.name;
        changed_data.name = "invalid-REQ-name".to_string();
        let new_artifacts = vec![changed_data.clone()];
        assert!(utils::split_artifacts(&p, &data_artifacts, &new_artifacts, false).is_err());
        changed_data.name = original;
    }
}

#[test]
fn test_update() {
    let simple = &test_data::TSIMPLE_DIR;
    let design = simple.join("design");

    let p = user::load_repo(simple.as_path()).unwrap();
    let data_artifacts: Vec<_> = p.artifacts
        .iter()
        .map(|(n, a)| a.to_data(&p.origin, n))
        .collect();

    let req_purpose = NameRc::from_str("REQ-purpose").unwrap();

    let mut changed_data = data_artifacts
        .iter()
        .filter(|a| a.name == "REQ-purpose")
        .next()
        .unwrap()
        .clone();

    // changing text is OK
    {
        let new_text = "changed to something else";
        changed_data.text = new_text.to_string();
        let new_artifacts = vec![changed_data.clone()];

        let new_project = crud::update_project(&data_artifacts, &p, &new_artifacts, false).unwrap();

        let mut expected = p.artifacts.get(&req_purpose).unwrap().clone();
        expected.text = new_text.to_string();
        expected.revision += 1;
        assert_eq!(new_project.artifacts.get(&req_purpose).unwrap(), &expected);
    }

    // changing path to new path NOT ok
    {
        let original = changed_data.def;
        changed_data.def = design.join("dne.toml").to_string_lossy().to_string();
        let new_artifacts = vec![changed_data.clone()];

        assert!(crud::update_project(&data_artifacts, &p, &new_artifacts, false).is_err());
        changed_data.def = original;
    }
}
