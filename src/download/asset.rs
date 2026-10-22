use std::path::Path;
use crate::download::{get, sha1, Download, DownloadSource};
use crate::model::asset::{AssetIndex, Index};
use crate::parse::Parse;

impl Download for AssetIndex {
    async fn download(&self, download_source: &DownloadSource, game_dir: &Path) -> Result<(), Box<dyn std::error::Error>> {
        println!("Downloading asset index");
        let indexes_dir = &game_dir.join("assets").join("indexes");
        if !indexes_dir.exists() {
            std::fs::create_dir_all(&indexes_dir)?;
        }

        let path = &indexes_dir.join(&format!("{}.json", self.id));
        std::fs::File::create(path)?;

        let url = &self.url;
        let text = &get(url).await?.text().await?;

        std::fs::write(path, text)?;

        let index = Index::parse(text)?;

        let objects_dir = &game_dir.join("assets").join("objects");

        if !objects_dir.exists() {
            std::fs::create_dir_all(objects_dir)?;
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
                if sha1(path)?.eq(hash) {
                    continue;
                } else {
                    std::fs::remove_file(path)?;
                }
            }

            std::fs::File::create(path)?;

            let url = match download_source {
                DownloadSource::Official => { format!("https://resources.download.minecraft.net/{}/{}", hash_first_two, hash) },
                DownloadSource::BMCLAPI => { format!("https://bmclapi2.bangbang93.com/assets/{}/{}", hash_first_two, hash) },
                DownloadSource::TSMCLAPI => { format!("https://tsmclapi.v1.dogteishou.cn/assets/{}/{}", hash_first_two, hash) },
                DownloadSource::Other(domain) => { format!("https://{}/{}/{}", domain, hash_first_two, hash) },
            };

            let bytes = &get(url).await?.bytes().await?;
            std::fs::write(path, bytes)?;
        }

        Ok(())
    }
}