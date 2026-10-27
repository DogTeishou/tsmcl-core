use std::path::Path;
use anyhow::bail;
use crate::download::{verify, Download, Source, sha1_of};
use crate::download::downloader::{DownloadTask, Downloader};
use crate::model::vanilla_version::VanillaVersion;

impl Download for VanillaVersion {
    async fn download(&self,
                      mut downloader: &mut Downloader,
                      source: &Source,
                      version_dir: &Path) -> anyhow::Result<()> {
        if !version_dir.exists() {
            bail!("No version directory found at {}", version_dir.display());
        }
        let name = version_dir.file_name().unwrap().to_str().unwrap();

        let game_dir = version_dir.parent().unwrap().parent().unwrap();

        if self.downloads.is_none() {
            bail!("No downloads found");
        }

        let url = source.get_version_url(&self.downloads.as_ref().unwrap().client.url);

        let client_path = &version_dir.join(format!("{}.jar", name));
        if client_path.exists() {
            if verify(&self.downloads.as_ref().unwrap().client.sha1, client_path) {
                return Ok(());
            }
        }

        let task = DownloadTask {
            url,
            path: client_path.clone(),
            sha1: Some(self.downloads.as_ref().unwrap().client.sha1.clone()),
        };

        downloader.add_task(task);

        Ok(())
    }
}