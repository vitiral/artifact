use super::schema::test_name;

#[derive(Insertable, Serialize)]
#[table_name="test_name"]
pub struct NewTestName<'a> {
    pub name: &'a str,
}

#[derive(Queryable)]
#[derive(Serialize, Deserialize)]
pub struct TestName {
    pub id: i32,
    pub name: String,
}
