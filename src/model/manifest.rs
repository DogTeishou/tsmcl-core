use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct VersionManifest {
    pub latest: Latest,
    pub versions: Vec<Version>,
}

#[derive(Deserialize, Serialize)]
pub struct Latest {
    pub release: String,
    pub snapshot: String,
}

#[derive(Deserialize, Serialize, Clone)]
pub struct Version {
    pub id: String,
    #[serde(alias = "type")]
    pub type_: String,
    pub url: String,
    pub time: String,
    #[serde(alias = "releaseTime")]
    pub release_time: String,
}