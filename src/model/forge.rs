use serde::Deserialize;

#[derive(Deserialize, Debug)]
pub struct Metadata {
    pub versioning: Versioning,
}

#[derive(Deserialize, Debug)]
pub struct Versioning {
    pub latest: Option<String>,
    pub release: Option<String>,
    pub versions: Vec<Version>,
}

#[derive(Deserialize, Debug)]
pub struct Version {
    #[serde(default)]
    pub version: Vec<String>,
}

impl Version {
    pub fn get_forge_version(&self, id: &str) -> Vec<String> {
        let mut versions = vec![];
        for v in &self.version {
            let splits: Vec<&str> = v.split("-").collect();
            let minecraft_version = splits[0];
            let forge_version = splits[1];
            if minecraft_version.eq(id) {
                versions.push(forge_version.to_string());
            }
        }
        versions
    }
}