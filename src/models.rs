use super::schema::test_name;

#[derive(Insertable)]
#[table_name="test_name"]
#[derive(Serialize)]
pub struct NewTestName {
    pub name: String,
}

#[derive(Queryable)]
#[derive(Serialize, Deserialize)]
pub struct TestName {
    pub id: i32,
    pub name: String,
}
