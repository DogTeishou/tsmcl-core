use std::path::Path;
use crate::download::{get, Download, ModLoader, Source};
use crate::download::downloader::{Downloader, Task};
use crate::model::asset::{AssetIndex, Index};
use crate::parse::Parse;

impl Download for AssetIndex {
    async fn build_task(&self, downloader: &mut Downloader, source: &Source, mod_loader: &ModLoader, game_dir: &Path) -> anyhow::Result<()> {
        println!("Downloading assets index {}...", self.id);
        let indexes_dir = &game_dir.join("assets").join("indexes");
        if !indexes_dir.exists() {
            tokio::fs::create_dir_all(&indexes_dir).await?;
        }

        let index_path = &indexes_dir.join(format!("{}.json", self.id));
        tokio::fs::File::create(index_path).await?;
        let url = self.url.clone();
        let text = get(url).await?.text().await?;

        tokio::fs::write(index_path, &text).await?;

        let index = Index::parse(&text)?;
        let objects_dir = &game_dir.join("assets").join("objects");
        if !objects_dir.exists() {
            tokio::fs::create_dir_all(&objects_dir).await?;
        }

        for (_, value) in index.objects {
            let hash = &value.hash;
            let hash_first_two = &hash[..2];

            let first_two_dir = &objects_dir.join(hash_first_two);
            if !first_two_dir.exists() {
                std::fs::create_dir_all(first_two_dir)?;
            }

            if !first_two_dir.exists() {
                std::fs::create_dir_all(first_two_dir)?;
            }

            let path = &first_two_dir.join(&hash);

            let url = match source {
                Source::Official => format!("https://resources.download.minecraft.net/{}/{}", hash_first_two, hash),
                Source::BMCLAPI => format!("https://api.bmclapi.com/minecraft_assets/{}/{}", hash_first_two, hash),
            };

            let task = Task::new(&url, path, Some(value.size as u64), Some(hash.clone()));
            downloader.add_task(task);

            std::fs::File::create(path)?;
        }

        Ok(())
    }
}