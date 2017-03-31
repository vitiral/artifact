

#[derive(Queryable, Serialize, Deserialize)]
pub struct TestName {
    pub id: i32,
    pub name: String,
}

table! {
	test_name (id) {
		id -> Int4,
		name -> Varchar,
	}
}