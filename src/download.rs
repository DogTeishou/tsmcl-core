pub mod version;
pub mod library;
pub mod asset;

use std::path::Path;
use sha1::Digest;

pub enum DownloadSource {
    Official,
    BMCLAPI,
    TSMCLAPI,
    Other(String)
}

pub trait Download {
    fn download(&self, download_source: &DownloadSource, game_dir: &Path) -> impl Future<Output=Result<(), Box<dyn std::error::Error>>>;
}

pub trait Allowed {
    fn allowed(&self) -> bool;
}

pub async fn get<T: reqwest::IntoUrl>(url: T) -> reqwest::Result<reqwest::Response> {
    reqwest::Client::builder()
        .timeout(std::time::Duration::from_secs(60))
        .build()?
        .get(url)
        .send().await
}

pub fn sha1<P: AsRef<Path>>(path: P) -> Result<String, std::io::Error> {
    let mut hasher = sha1::Sha1::new();
    file_hashing::get_hash_file(path, &mut hasher)
}