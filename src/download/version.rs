use std::path::Path;
use crate::download::{get, Download, ModLoader, Source};
use crate::download::downloader::{Downloader, Task};
use crate::model::version_manifest::Version;
use crate::parse::Parse;

impl Download for Version {
    async fn build_task(&self, downloader: &mut Downloader, source: &Source, mod_loader: &ModLoader, game_dir: &Path) -> anyhow::Result<()> {
        println!("Downloading version {}", self.id);

        let game = crate::model::version::Version::parse(get(&self.url).await?.text().await?.as_str())?;
        let version_dir = &game_dir.join("versions").join(&game.id);

        if !version_dir.exists() {
            std::fs::create_dir_all(&version_dir)?;
        }

        game.libraries.build_task(downloader, source,  mod_loader, game_dir).await?;
        game.asset_index.build_task(downloader, source, mod_loader, game_dir).await?;

        let version_config = &game_dir
            .join("versions")
            .join(&game.id)
            .join(&format!("{}.json", &self.id));

        if version_config.exists() {
            std::fs::remove_file(version_config)?;
        }

        std::fs::File::create(version_config)?;
        let url = game.downloads.client.url.clone();

        let path = &game_dir
            .join("versions")
            .join(&game.id)
            .join(&format!("{}.jar", &game.id));

        std::fs::File::create(path)?;

        let task = Task::new(&url, path, Some(game.downloads.client.size as u64), None);
        downloader.add_task(task);

        Ok(())
    }
}
