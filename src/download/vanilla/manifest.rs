use std::path::Path;

use anyhow::{Ok, bail};

use crate::{download::{Source, get}, model::{manifest::{Version, VersionManifest}, vanilla_version::VanillaVersion}, parse::Parse};

impl VersionManifest {
    pub async fn get_manifest(source: &Source) -> anyhow::Result<Self> {
        let url = source.get_manifest_url();
        let text = get(url).await?.text().await?;
        let manifest = VersionManifest::parse(&text)?;
        Ok(manifest)
    }

    pub async fn get_version(&self, id: &str, name: &str) -> anyhow::Result<Version> {
        let mut version= Version { 
            id: String::from("null"), 
            type_: String::new(), 
            url: String::new(), 
            time: String::new(), 
            release_time: String::new() 
        };

        for v in &self.versions {
            if v.id.eq(id) {
                version = v.clone();
                break;
            }
        }

        if version.id.eq("null") {
            bail!("version id unknown");
        }

        version.id = name.to_string();

        Ok(version)
    }
}

impl Version {
    pub async fn download(&self, source: &Source, game_dir: &Path) -> anyhow::Result<VanillaVersion> {
        let version_dir = &game_dir.join("versions").join(format!("{}", self.id));
        let config_path = &version_dir.join(format!("{}.json", self.id));
        if !version_dir.exists() {
            tokio::fs::create_dir_all(version_dir).await?;
        }

        if !config_path.exists() {
            tokio::fs::File::create(config_path).await?;
        }

        let url = source.get_version_url(&self.url);
        let text = get(url).await?.text().await?;

        let mut config = VanillaVersion::parse(&text)?;

        if let Some(arguments) = &mut config.arguments {
            for argument in &mut arguments.game {
                argument.set_all_features_false();
            }
        }

        let text = serde_json::to_string_pretty(&config)?;

        tokio::fs::write(config_path, text).await?;

        Ok(config)
    }
}