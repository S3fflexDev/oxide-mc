use serde::{Serialize, Deserialize};

#[derive(Debug, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct FabricProfile {
    pub id: String,
    pub inherits_from: String,
    pub release_time: String,
    pub time: String,
    #[serde(rename = "type")]
    pub profile_type: String,
    pub main_class: String,
    pub arguments: FabricArguments,
    pub libraries: Vec<FabricLibrary>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FabricArguments {
    pub game: Vec<serde_json::Value>,
    pub jvm: Vec<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct FabricLibrary {
    pub name: String,
    pub url: String,
    pub md5: Option<String>,
    pub sha1: Option<String>,
    pub sha256: Option<String>,
    pub sha512: Option<String>,
    pub size: Option<i64>,
}