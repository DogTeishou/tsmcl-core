use serde::{Deserialize, Serialize};

#[derive(Deserialize, Serialize)]
pub struct Library {
    pub downloads: Download,
    pub name: String,
    pub rules: Option<Vec<Rule>>,
}

#[derive(Deserialize, Serialize)]
pub struct Rule {
    pub action: String,
    pub os: Os,
}

#[derive(Deserialize, Serialize)]
pub struct Os {
    pub name: String,
}

#[derive(Deserialize, Serialize)]
pub struct Download {
    pub artifact: Artifact,
}

#[derive(Deserialize, Serialize)]
pub struct Artifact {
    pub path: String,
    pub sha1: String,
    pub size: u32,
    pub url: String,
}