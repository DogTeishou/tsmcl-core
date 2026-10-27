use std::fmt::format;
use std::path::Path;
use crate::download::{get, Download, Downloader, Source, DownloadTask, verify};
use crate::model::asset::{AssetIndex, Index};
use crate::parse::Parse;

impl Download for AssetIndex {
    async fn download(&self, downloader: &mut Downloader, source: &Source, game_dir: &Path) -> anyhow::Result<()> {
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
            if path.exists() {
                if verify(hash, path) {
                    continue;
                }
            }

            let url = source.get_asset_url(&format!("https://resources.download.minecraft.net/{}/{}", hash_first_two, hash));

            let task = DownloadTask {
                url,
                path: path.to_path_buf(),
                sha1: Some(hash.clone()),
            };

            downloader.add_task(task);
        }

        Ok(())
    }
}