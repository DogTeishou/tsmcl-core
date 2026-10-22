use std::collections::HashMap;
use serde::Deserialize;

#[derive(Deserialize)]
pub struct AssetIndex {
    pub id: String,
    pub size: u32,
    pub sha1: String,
    #[serde(alias = "totalSize")]
    pub total_size: u32,
    pub url: String,
}

#[derive(Deserialize)]
pub struct Index {
    pub objects: HashMap<String, Object>,
}

#[derive(Deserialize)]
pub struct Object {
    pub hash: String,
    pub size: u32,
}