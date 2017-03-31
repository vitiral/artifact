#[derive(Queryable)]
#[derive(Serialize, Deserialize)]
pub struct TestName {
    pub id: i32,
    pub name: String,
}