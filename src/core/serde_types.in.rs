
#[derive(Serialize, Deserialize, Debug, Eq, PartialEq)]
pub struct LocData {
    pub path: String,
    pub row: u64,
    pub col: u64,
}

#[derive(Serialize, Deserialize, Debug, PartialEq)]
pub struct ArtifactData {
    pub id: u64,
    pub name: String,
    pub text: String,
    pub partof: Vec<String>,
    pub parts: Vec<String>,
    pub loc: LocData,
    pub completed: f32,
    pub tested: f32,
}
