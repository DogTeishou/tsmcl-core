use std::{collections::HashMap, path::Path};
use std::io::Read;
use reqwest::{IntoUrl, Response};
use sha1::Digest;

pub mod vanilla;
pub mod downloader;

pub use downloader::*;

#[cfg(test)]
pub mod tests;


pub enum Source {
    Official,
    BMCLAPI,
    Custom(HashMap<String, String>),
}

pub trait Download {
    fn download(&self,
                downloader: &mut Downloader,
                source: &Source,
                game_dir: &Path) -> impl Future<Output=anyhow::Result<()>>;
}

pub async fn get<U: IntoUrl>(url: U) -> Result<Response, reqwest::Error> {
    reqwest::Client::builder().build()?.get(url).send().await
}

fn sha1_of(path: &Path) -> anyhow::Result<String> {
    let mut file = std::fs::File::open(path)?;
    let mut hasher = sha1::Sha1::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = file.read(&mut buf)?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}

pub fn verify(sha1: &String, path: &Path) -> bool {
    let mut verified = false;
    let file_sha1 = match sha1_of(path) {
        Ok(sha1) => sha1,
        Err(_) => return verified,
    };

    if *sha1 == file_sha1 {
        verified = true;
    }

    verified
}

impl Source {
    pub fn get_manifest_url(&self) -> String {
        match self {
            Source::Official => "https://piston-meta.mojang.com/mc/game/version_manifest.json".to_string(),
            Source::BMCLAPI => "https://bmclapi2.bangbang93.com/mc/game/version_manifest_v2.json".to_string(),
            Source::Custom(url) => url["https://piston-meta.mojang.com/mc/game/version_manifest.json"].clone(),
        }
    }

    pub fn get_version_url(&self, url: &String) -> String {
        match self {
            Source::Official => url.to_string(),
            Source::BMCLAPI => url.replace("piston-meta.mojang.com", "bmclapi2.bangbang93.com"),
            Source::Custom(uri) => url.replace("piston-meta.mojang.com", &uri["piston-meta.mojang.com"]),
        }
    }

    pub fn get_asset_url(&self, url: &String) -> String {
        match self {
            Source::Official => url.to_string(),
            Source::BMCLAPI => url.replace("resources.download.minecraft.net", "bmclapi2.bangbang93.com/assets"),
            Source::Custom(uri) => url.replace("resources.download.minecraft.net", &uri["resources.download.minecraft.net"]),
        }
    }

    pub fn get_library_url(&self, url: &String) -> String {
        match self {
            Source::Official => url.to_string(),
            Source::BMCLAPI => url.replace("libraries.minecraft.net", "bmclapi2.bangbang93.com/maven"),
            Source::Custom(uri) => url.replace("libraries.minecraft.net", &uri["libraries.minecraft.net"]),
        }
    }
}