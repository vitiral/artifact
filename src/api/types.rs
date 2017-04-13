#[derive(Queryable, Insertable, Serialize, Deserialize)]
#[table_name="test_name"]
pub struct TestName {
    pub name: String,
}

#[derive(Queryable, Insertable)]
#[table_name="artifact_name"]
pub struct ArtifactName {
	pub name: String,
}

#[derive(Queryable, Insertable)]
#[table_name="version"]
pub struct Version {
	pub id: i32,
	pub major: String,
	pub minor: Option<String>,
	pub patch: Option<String>,
	pub build: Option<String>,
}

#[derive(Debug, Queryable, Insertable, Serialize)]
#[table_name="test_run"]
pub struct TestRun {
	pub id: i32,
	pub test_name: String,
	pub passed: bool,
	pub artifacts: Vec<String>,
	pub date: String,	// what goes here?
	pub version: i32,
	pub link: Option<String>,
	pub data: Option<String>,	// what goes here?
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="test_run"]
pub struct NewTestRun {
	pub name: String,
	pub passed: bool,
	pub artifacts: Vec<String>,
	pub date: String,	
	pub version: i32,
	pub link: Option<String>,
	pub data: Option<String>,
}


table! {
	test_name (name) {
		name -> Text,
	}
}

table! {
	artifact_name (name) {
		name -> Text,
	}
}

table! {
	version (id) {
		id -> Int4,
		major -> Text,
		minor -> Nullable<Text>,
		patch -> Nullable<Text>,
		build -> Nullable<Text>,
	}
}

table! {
	test_run (id) {
		id -> Int4,
		test_name -> Text,
		passed -> Bool,
		artifacts -> Array<Text>,
		date -> Text,
		version -> Int4,
		link -> Nullable<Text>,
		data -> Nullable<Text>,
	}
}