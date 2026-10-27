use std::path::Path;
use crate::download::{verify, Download, DownloadTask, Source};
use crate::download::downloader::Downloader;
use crate::model::Allowed;
use crate::model::vanilla_version::Libraries;

impl Download for Libraries {
    async fn download(&self,
                      downloader: &mut Downloader,
                      source: &Source,
                      game_dir: &Path) -> anyhow::Result<()> {
        let libraries_dir = &game_dir.join("libraries");
        if !libraries_dir.exists() {
            std::fs::create_dir_all(libraries_dir)?;
        }
        for library in self {
            if !library.allowed() {
                continue;
            }
            if let Some(artifact) = &library.downloads.artifact {
                let lib_path = &libraries_dir.join(&artifact.path);
                if lib_path.exists() {
                    if !verify(&artifact.sha1, lib_path) {
                        std::fs::remove_file(lib_path)?;
                    }
                }
                if !lib_path.parent().unwrap().exists() {
                    std::fs::create_dir_all(&lib_path.parent().unwrap())?;
                }
                let url = source.get_library_url(&artifact.url);
                let task = DownloadTask {
                    url,
                    path: lib_path.to_path_buf(),
                    sha1: Some(artifact.sha1.clone()),
                };
                downloader.add_task(task);
            }
        }

        for lib in self {
            if !lib.allowed() {
                continue;
            }
            if let Some(native) = lib.get_natives() {
                if let Some(download) = lib
                    .downloads
                    .classifiers
                    .as_ref()
                    .and_then(|classifiers| classifiers.get(&native))
                {
                    let url = source.get_library_url(&download.url);
                    let lib_path = &libraries_dir.join(&download.path);
                    if lib_path.exists() && !verify(&download.sha1, lib_path) {
                        std::fs::remove_file(lib_path)?;
                    }
                    let task = DownloadTask {
                        url,
                        path: lib_path.clone(),
                        sha1: Some(download.sha1.clone()),
                    };
                    downloader.add_task(task);
                }
            }
        }

        Ok(())
    }
}