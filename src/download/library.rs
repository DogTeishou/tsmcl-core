use std::path::Path;
use crate::download::{Download, ModLoader, Source};
use crate::download::downloader::{Downloader, Task};
use crate::model::library::Library;
use crate::model::version::Libraries;

pub trait Allowed {
    fn allowed(&self) -> bool;
}

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
    async fn build_task(&self, downloader: &mut Downloader, source: &Source, mod_loader: &ModLoader, game_dir: &Path) -> anyhow::Result<()> {
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

            let task = Task::new(&lib.downloads.artifact.url, lib_path, Some(lib.downloads.artifact.size as u64), Some(lib.downloads.artifact.sha1.clone()));
            downloader.add_task(task);
        }

        Ok(())
    }
}