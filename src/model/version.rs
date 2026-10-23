use serde::{Deserialize, Serialize};
use crate::model::library::Library;
use crate::model::asset::AssetIndex;

pub type Libraries = Vec<Library>;

#[derive(Deserialize, Serialize)]
pub struct Version {
    pub id: String,
    #[serde(alias = "assetIndex")]
    pub asset_index: AssetIndex,
    pub downloads: Downloads,
    pub libraries: Libraries,
    #[serde(alias = "mainClass")]
    pub main_class: String,
    #[serde(alias = "releaseTime")]
    pub release_time: String,
    pub time: String,
    #[serde(alias = "type")]
    pub type_: String,
}

#[derive(Deserialize, Serialize)]
pub struct Downloads {
    pub client: MainJar,
    pub server: MainJar,
}

#[derive(Deserialize, Serialize)]
pub struct MainJar {
    pub sha1: String,
    pub size: u32,
    pub url: String,
}