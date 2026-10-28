use std::thread;
use crate::{download::{Download, Source}, model::manifest::VersionManifest};
use crate::download::downloader::{DownloadTask, Downloader};
use crate::download::downloader::progressor::terminal_sink;

#[test]
fn test_version_1_12_2() -> anyhow::Result<()> {
    download_version("1.12.2")
}

#[test]
fn test_version_1_18() -> anyhow::Result<()> {
    download_version("1.18")
}

#[test]
fn test_version_26_3() -> anyhow::Result<()> {
    download_version("26.3")
}

#[test]
fn test_version_rd_132211() -> anyhow::Result<()> {
    download_version("rd-132211")
}

#[test]
fn test_version_1_0() -> anyhow::Result<()> {
    download_version("1.0")
}

fn download_version(version: &str) -> anyhow::Result<()> {
    let mut vd = Downloader::new(vec![], 32, 1024 * 1024, 5 * 1024 * 1024)?
        .on_progress(terminal_sink);

    let mut ad = Downloader::new(vec![], 16, 1024 * 1024, 5 * 1024 * 1024)?
        .on_progress(terminal_sink);

    let mut ld = Downloader::new(vec![], 16, 1024 * 1024, 5 * 1024 * 1024)?
        .on_progress(terminal_sink);

    let runtime = tokio::runtime::Runtime::new()?;
    runtime.block_on(async {
        let game_dir = std::env::current_dir()?.join(".minecraft");
        let version_dir = game_dir.join("versions").join(version);

        let v = VersionManifest::get_manifest(&Source::Official)
            .await?
            .get_version(version, version)
            .await?
            .download(&Source::Official, &game_dir)
            .await?;
        if let Some(l) = &v.libraries {
            l.download(&mut ld, &Source::Official, &game_dir).await?;
        }
        if let Some(a) = &v.asset_index {
            a.download(&mut ad, &Source::Official, &game_dir).await?;
        }
        v.download(&mut vd, &Source::Official, &version_dir).await?;

        Ok::<(), anyhow::Error>(())
    })?;
    drop(runtime);

    vd.run()?;
    ad.run()?;
    ld.run()?;
    Ok(())
}

#[test]
fn test_downloader() -> anyhow::Result<()> {
    let game_dir = &std::env::current_dir()?.join(".minecraft").join("downloads");
    if !game_dir.exists() {
        std::fs::create_dir_all(game_dir)?;
    }
    let download_dir = &game_dir.join("1GB.zip");
    if !download_dir.exists() {
        std::fs::File::create(download_dir)?;
    }
    let tasks = vec![DownloadTask {
        url: "http://xcal1.vodafone.co.uk/1GB.zip".to_string(),
        path: download_dir.clone(),
        sha1: None,
        size: None
    }];
    let downloader = Downloader::new(
        tasks,
        32,
        1024*1024*4,
        5 *1024*1024)?
        .on_progress(terminal_sink);
    println!("Downloading...");
    let t = thread::spawn(move || downloader.run());
    t.join().ok();
    Ok(())
}
