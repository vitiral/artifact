#[derive(Serialize, Deserialize, Debug, Clone, Eq, PartialEq)]
pub struct LocData {
    pub path: String,
    pub row: u64,
    pub col: u64,
}

/// the Text object contains both the raw and the resolved
/// text value (after variable resolution)
#[derive(Serialize, Deserialize, Clone, Debug, PartialEq)]
pub struct Text {
    pub raw: String,
    pub value: String,
}

#[derive(Serialize, Deserialize, Debug, Clone, PartialEq)]
pub struct ArtifactData {
    pub id: u64,
    pub name: String,
    pub path: String,
    pub text: Text,
    pub partof: Vec<String>,

    // // TODO: until I serde gets up to speed, the web-api will
    // // have to send these values even though they are ignored
    //#[serde(default)]
    pub parts: Vec<String>,
    //#[serde(default)]
    pub loc: Option<LocData>,
    //#[serde(default = -1)]
    pub completed: f32,
    //#[serde(default = -1)]
    pub tested: f32,
}

pub enum RpcErrors {
    xIdsNotFound,
    xFilesNotFound,
    xInvalidName,
    xInvalidPartof,
}
