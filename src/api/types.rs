

#[derive(Queryable, Serialize, Deserialize, Identifiable)]
pub struct TestName {
    pub id: i32,
    pub name: String,
}
/*
#[derive(Insertable)]
#[table_name="test_names"]
pub struct NewTestName {
	pub name: String,
}

#[derive(Queryable, Insertable)]
#[table_name="artifact_names"]
pub struct ArtifactName {
	pub name: String,
}

#[derive(Queryable, Insertable)]
#[table_name="version"]
pub struct Version {
	pub id: i32,
	pub major: Option<String>,
	pub minor: Option<String>,
	pub patch: Option<String>,
	pub build: Option<String>,
}

#[derive(Queryable, Insertable)]
#[table_name="test_info"]
pub struct TestInfo {
	pub id: i32,
	pub test_name: String,
	pub passed: boolean,
	pub artifacts: Vec<String>,
	//pub date: ,	// what goes here?
	pub version: i32,
	pub link: Option<String>,
	pub data: Option< >,	// what goes here?
}
*/
table! {
	test_names (id) {
		id -> Int4,
		name -> Text,
	}
}

table! {
	artifact_names (name) {
		name -> Text,
	}
}
/*
table! {
	version (id) {
		id -> Int4,
		major -> Nullable<Text>,
		minor -> Nullable<Text>,
		patch -> Nullable<Text>,
		build -> Nullable<Text>,
	}
}

table! {
	test_info (id) {
		id -> Int4,
		test_name -> Text,
		passed -> Bool,
		artifacts -> Array<Text>,
		date -> Timestamp,
		version -> Int4
		link -> Nullable<text>,
		data -> Nullable<Array<Bytea>>,
	}
}*/