use std::path::Path;
use crate::download::{get, Download, ModLoader, Source};
use crate::download::downloader::{Downloader, Task};
use crate::model::version_manifest::Version;
use crate::parse::Parse;

impl Download for Version {
    async fn build_task(&self, downloader: &mut Downloader, source: &Source, mod_loader: &ModLoader, game_dir: &Path) -> anyhow::Result<()> {
        println!("Downloading version {}", self.id);

        let text = get(self.url.clone()).await?.text().await?;

        let game = crate::model::version::Version::parse(text.as_str())?;

        let version_dir = &game_dir.join("versions").join(&game.id);

        if !version_dir.exists() {
            std::fs::create_dir_all(&version_dir)?;
        }

        game.libraries.build_task(downloader, source,  mod_loader, game_dir).await?;
        game.asset_index.build_task(downloader, source, mod_loader, game_dir).await?;

        let url = game.downloads.client.url.clone();

        let path = &game_dir
            .join("versions")
            .join(&game.id)
            .join(&format!("{}.jar", &game.id));

        std::fs::File::create(path)?;

        let version_config = &game_dir
            .join("versions")
            .join(&self.id)
            .join(&format!("{}.json", &self.id));

        let string = serde_json::to_string(&game)?;
        if version_config.exists() {
            tokio::fs::remove_file(version_config).await?;
        }

        tokio::fs::write(version_config, string).await?;

        let task = Task::new(&url, path, Some(game.downloads.client.size as u64), None);
        downloader.add_task(task);

        Ok(())
    }
}
