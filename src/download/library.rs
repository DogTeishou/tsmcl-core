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
                    if verify(&artifact.sha1, lib_path) {
                        continue;
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

            if let Some(native) = &library.get_natives() {
                if let Some(classifiers) = &library.downloads.classifiers {
                    let di = &classifiers[native];
                    let lib_path = &libraries_dir.join(&di.path);
                    if verify(&di.sha1, lib_path) {
                        continue;
                    }
                    if !lib_path.parent().unwrap().exists() {
                        std::fs::create_dir_all(&lib_path.parent().unwrap())?;
                    }
                    let task = DownloadTask {
                        url: di.url.clone(),
                        path: lib_path.to_path_buf(),
                        sha1: Some(di.sha1.clone()),
                    };
                    downloader.add_task(task);
                }
            }
        }

        Ok(())
    }
}