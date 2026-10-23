pub mod version;
pub mod downloader;
pub mod library;
pub mod asset;

use std::path::Path;
use crate::download::downloader::Downloader;

pub enum ModLoader {
    Vanilla,
    Forge,
    NeoForge,
    Fabric,
    Quilt,
}

pub enum Source {
    Official,
    BMCLAPI
}

pub trait Download {
    fn build_task(&self, downloader: &mut Downloader, source: &Source, mod_loader: &ModLoader, game_dir: &Path) -> impl Future<Output = anyhow::Result<()>> + Send;
}

pub async fn get<T: reqwest::IntoUrl>(url: T) -> reqwest::Result<reqwest::Response> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?
        .get(url)
        .send().await
}

#[cfg(test)]
mod tests {
    use std::sync::Arc;
    use crate::download::{Download, ModLoader, Source};
    use crate::download::downloader::{DownloadOptions, Downloader};
    use crate::model::version_manifest::Version;

    #[tokio::test]
    async fn test_download() -> anyhow::Result<()> {
        let version = Version {
            id: "26.3".to_string(),
            type_: "release".to_string(),
            url: "https://piston-meta.mojang.com/v1/packages/61fff1316bd50312624a858bbb5eb18fa45208ed/26.3.json".to_string(),
            time : "2026-10-20T06:46:34+00:00".to_string(),
            release_time : "2026-09-15T11:23:02+00:00".to_string(),
        };

        let download_path = &std::env::current_dir()?.join(".minecraft");
        std::fs::create_dir_all(download_path).unwrap_or_else(|err| panic!("{:?}", err));

        let mut downloader = Downloader::new(DownloadOptions::default())?;
        version.build_task(&mut downloader, &Source::Official, &ModLoader::Vanilla, download_path).await?;

        let downloader = Arc::new(downloader);
        let _vec = downloader.download_all().await;

        Ok(())
    }
}