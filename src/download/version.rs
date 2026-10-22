use std::path::Path;
use crate::download::{get, sha1, Download, DownloadSource};
use crate::model::version_manifest::Version;
use crate::parse::Parse;

impl Download for Version {
    async fn download(&self, download_source: &DownloadSource, game_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("Downloading version {}", self.id);
        let url: &str = match download_source {
            DownloadSource::Official => { &self.url },
            DownloadSource::BMCLAPI => { &self.url.replace("piston-meta.mojang.com", "bmclapi2.bangbang93.com") },
            DownloadSource::TSMCLAPI => { &self.url.replace("piston-meta.mojang.com", "tsmclapi.v1.dogteishou.cn") },
            DownloadSource::Other(domain) => { &self.url.replace("piston-meta.mojang.com", domain.as_str()) },
        };

        let game = crate::model::version::Version::parse(get(url).await?.text().await?.as_str())?;
        let version_dir = &game_dir.join("versions").join(&game.id);

        if !version_dir.exists() {
            std::fs::create_dir_all(&version_dir)?;
        }

        game.libraries.download(download_source, game_dir).await?;
        game.asset_index.download(download_source, game_dir).await?;

        let version_config = &game_dir
            .join("versions")
            .join(&game.id)
            .join(&format!("{}.json", &self.id));

        if version_config.exists() {
            std::fs::remove_file(version_config)?;
        }

        std::fs::File::create(version_config)?;
        std::fs::write(version_config, get(&self.url).await?.bytes().await?)?;

        let path = &game_dir
            .join("versions")
            .join(&game.id)
            .join(&format!("{}.jar", &game.id));

        if path.exists() {
            if sha1(path)?.eq(&game.downloads.client.sha1) {
                return Ok(());
            } else {
                std::fs::remove_file(path)?;
            }
        }

        std::fs::File::create(path)?;

        let bytes = get(&game.downloads.client.url).await?.bytes().await?;

        std::fs::write(path, bytes)?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_download() -> Result<(), Box<dyn std::error::Error>> {
        let version = Version {
            id: "1.21".to_string(),
            type_: "release".to_string(),
            url: "https://piston-meta.mojang.com/v1/packages/177e49d3233cb6eac42f0495c0a48e719870c2ae/1.21.json".to_string(),
            time : "2024-06-13T08:32:38+00:00".to_string(),
            release_time : "2024-06-13T08:24:03+00:00".to_string(),
        };

        let download_path = &std::env::current_dir()?.join("rust-minecraft-client-launch");
        std::fs::create_dir_all(download_path).unwrap_or_else(|err| panic!("{:?}", err));

        if let Err(err) = version.download(&DownloadSource::Official, download_path).await {
            panic!("{:?}", err);
        }
        Ok(())
    }
}