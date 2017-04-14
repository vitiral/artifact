extern crate diesel;


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
	pub epoch: f32,
	pub version_id: i32,
	pub link: Option<String>,
	pub data: Option<Vec<u8>>,
}

#[derive(Debug, Insertable, Serialize, Deserialize)]
#[table_name="test_run"]
pub struct NewTestRun {
	pub test_name: String,
	pub passed: bool,
	pub artifacts: Vec<String>,
	pub epoch: f32,	
	pub version_id: i32,
	pub link: Option<String>,
	pub data: Option<Vec<u8>>,
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
		epoch -> Float,
		version_id -> Int4,
		link -> Nullable<Text>,
		data -> Nullable<Binary>,
	}
}