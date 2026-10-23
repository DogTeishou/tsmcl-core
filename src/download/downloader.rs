use std::path::{Path, PathBuf};
use std::sync::Arc;
use std::sync::atomic::{AtomicU64, AtomicUsize};
use std::sync::atomic::Ordering::Relaxed;
use std::time::Duration;
use std::sync::Mutex;
use anyhow::{bail, Context};
use futures::StreamExt;
use sha1::Digest;
use tokio::fs::OpenOptions;
use tokio::io::{AsyncReadExt, AsyncWriteExt};
use tokio::sync::Semaphore;

pub struct DownloadOptions {
    pub threads: usize,
    pub chunk_threshold: u64,
    pub chunk_size: u64,
    pub retries: usize,
}

impl Default for DownloadOptions {
    fn default() -> Self {
        Self {
            threads: 32,
            chunk_threshold: 1024 * 1024 * 16,
            chunk_size: 1024 * 1024 * 4,
            retries: 3,
        }
    }
}

#[derive(Debug, Clone)]
pub struct Task {
    pub url: String,
    pub path: PathBuf,
    pub size: Option<u64>,
    pub sha1: Option<String>,
}

impl Task {
    pub fn new(url: &String, path: &PathBuf, size: Option<u64>, sha1: Option<String>) -> Self {
        Self { url: url.clone(), path: path.clone(), size, sha1 }
    }
}

#[derive(Default)]
pub struct Progress {
    pub files_total: AtomicUsize,
    pub files_done: AtomicUsize,
    pub files_failed: AtomicUsize,
    pub bytes_total: AtomicU64,
    pub bytes_done: AtomicU64,
}

pub struct Downloader {
    tasks: Vec<Task>,
    client: reqwest::Client,
    semaphore: Arc<Semaphore>,
    limit: Mutex<usize>,
    options: DownloadOptions,
    pub progress: Arc<Progress>,
}

const MAX_THREADS: usize = 256;

impl Downloader {
    pub fn new(options: DownloadOptions) -> anyhow::Result<Self> {
        let n = options.threads.clamp(1, MAX_THREADS);
        let client = reqwest::Client::builder()
            .pool_max_idle_per_host(n)
            .connect_timeout(Duration::from_secs(15))
            .read_timeout(Duration::from_secs(30))
            .build()?;
        let semaphore = Arc::new(Semaphore::new(n));
        let limit = Mutex::new(n);
        let progress = Arc::new(Progress::default());
        Ok(Self { tasks: vec![], client, semaphore, limit, options, progress })
    }

    pub fn set_threads(&self, n: usize) {
        let n = n.clamp(1, MAX_THREADS);
        let mut cur = self.limit.lock().unwrap();
        if n > *cur {
            self.semaphore.add_permits(n - *cur);
        } else {
            self.semaphore.forget_permits(*cur - n);
        }
        *cur = n;
    }

    pub fn add_task(&mut self, task: Task) {
        self.tasks.push(task);
    }

    pub async fn download_all(self: &Arc<Self>) -> Vec<(String, anyhow::Error)> {
        let progress = &self.progress;
        progress.files_total.store(self.tasks.len(), Relaxed);
        progress.bytes_total.store(self.tasks.iter().filter_map(|t| t.size).sum(), Relaxed);

        futures::stream::iter(self.tasks.clone())
            .map(|t| {
                let this = self.clone();
                tokio::spawn( async move {
                    let result = this.download_one(&t).await;
                    match &result {
                        Ok(_) => this.progress.files_done.fetch_add(1, Relaxed),
                        Err(_) => this.progress.files_failed.fetch_add(1, Relaxed),
                    };
                    result.err().map(|e| (t.url, e))
                })
            })
            .buffer_unordered(self.options.threads)
            .filter_map(|r| async move {
                r.ok().flatten()
            }).collect().await
    }

    pub async fn download_one(&self, task: &Task) -> anyhow::Result<()> {
        let _permit = self.semaphore.acquire().await?;
        let res = self.client
            .get(&task.url)
            .send()
            .await?;
        if task.path.exists() {
            tokio::fs::remove_file(&task.path).await?;
        }
        let mut file = tokio::fs::File::create(&task.path).await?;
        let mut body = res.bytes_stream();
        while let Some(chunk) = body.next().await {
            let chunk = chunk?;
            file.write_all(&chunk).await?;
            self.progress.bytes_done.fetch_add(chunk.len() as u64, Relaxed);
        }
        file.flush().await?;
        drop(file);
        self.verify(task, task.path.as_path()).await?;
        Ok(())
    }

    async fn verify(&self, task: &Task, path: &Path) -> anyhow::Result<()> {
        if let Some(size) = task.size {
            let len = tokio::fs::metadata(path).await?.len();
            if len != size {
                bail!("file size mismatch: expected {}, got {}", size, len);
            }
        }
        if let Some(sha1) = &task.sha1 {
            let got = sha1_of(path).await?;
            if !got.eq_ignore_ascii_case(sha1) {
                bail!("sha1 mismatch: expected {}, got {}", sha1, got);
            }
        }
        Ok(())
    }
}

async fn sha1_of(path: &Path) -> anyhow::Result<String> {
    let mut f = tokio::fs::File::open(path).await?;
    let mut hasher = sha1::Sha1::new();
    let mut buf = [0u8; 64 * 1024];
    loop {
        let n = f.read(&mut buf).await?;
        if n == 0 {
            break;
        }
        hasher.update(&buf[..n]);
    }
    Ok(hex::encode(hasher.finalize()))
}