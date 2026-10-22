use std::error::Error;
use std::path::Path;
use crate::download::{get, sha1, Allowed, Download, DownloadSource};
use crate::model::library::Library;
use crate::model::version::Libraries;

impl Allowed for Library {
    fn allowed(&self) -> bool {
        let mut allowed = true;

        if self.rules.is_some() {
            for rule in self.rules.as_ref().unwrap()  {
                if rule.os.name == "osx" && !cfg!(target_os = "macos") {
                    allowed = false;
                    break;
                } else if rule.os.name == "windows" && !cfg!(target_os = "windows") {
                    allowed = false;
                    break;
                } else if rule.os.name == "linux" && !cfg!(target_os = "linux") {
                    allowed = false;
                    break;
                }
            }
        }

        if self.name.contains("natives") {
            if self.name.contains("x86") && !cfg!(target_arch = "x86") {
                allowed = false;
            } else if self.name.contains("arm64") && !cfg!(target_arch = "aarch64") {
                allowed = false;
            } else if !cfg!(target_arch = "x86_64") {
                allowed = false;
            }
        }

        allowed
    }
}

impl Download for Libraries {
    async fn download(&self, download_source: &DownloadSource, game_dir: &Path) -> Result<(), Box<dyn Error>> {
        println!("Downloading libraries");
        let libraries_dir = &game_dir.join("libraries");

        if !libraries_dir.exists() {
            std::fs::create_dir_all(&libraries_dir)?;
        }

        for lib in self {
            if !lib.allowed() {
                continue;
            }

            let lib_file = &lib.downloads.artifact.path;
            let lib_path = &libraries_dir.join(lib_file);

            if !lib_path.parent().unwrap().exists() {
                std::fs::create_dir_all(&lib_path.parent().unwrap())?;
            }

            if lib_path.exists() {
                if sha1(lib_path)?.eq(&lib.downloads.artifact.sha1) {
                    continue;
                } else {
                    std::fs::remove_file(lib_path)?;
                }
            }

            let url: &str = match &download_source {
                DownloadSource::Official => { &lib.downloads.artifact.url },
                DownloadSource::BMCLAPI => { &lib.downloads.artifact.url.replace("libraries.minecraft.net", "bmclapi2.bangbang93.com/maven") },
                DownloadSource::TSMCLAPI => { &lib.downloads.artifact.url.replace("libraries.minecraft.net", "tsmclapi.v1.dogteishou.cn/libs") },
                DownloadSource::Other(domain) => { &lib.downloads.artifact.url.replace("libraries.minecraft.net", domain.as_str()) },
            };

            let bytes = &get(url).await?.bytes().await?;
            std::fs::write(lib_path, bytes)?;
        }

        Ok(())
    }
}