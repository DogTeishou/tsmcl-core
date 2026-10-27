use std::io::Read;
use std::io::Write;
use std::error::Error;
use std::path::PathBuf;
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};
use std::time::Duration;
use anyhow::bail;
use reqwest::{blocking::Client, StatusCode, header::{CONTENT_RANGE, RANGE}};
use std::fs::{File, OpenOptions};
use std::thread;
use crate::download::downloader::progressor::{Progressor, Snapshot, Status};
use crate::download::verify;

pub mod progressor;

/// 小文件阈值：已知大小 <= min(chunk_size, SMALL_FILE_LIMIT) 的文件，
/// 不走分块（省去 Range 请求和预分配），一次 GET 读入内存后整体写盘。
const SMALL_FILE_LIMIT: usize = 4 * 1024 * 1024;

enum SubTask {
    /// 某个文件的其中一个分块
    Chunk {
        url: String,
        path: PathBuf,
        file: Arc<File>,
        start: usize,
        end: usize,
        abort: Arc<AtomicBool>,
        chunk_left: Arc<AtomicUsize>,
        sha1: Option<String>,
    },
    /// 小文件：单次 GET，整体落盘，支持整体重试
    Small {
        url: String,
        path: PathBuf,
        size: usize,
        sha1: Option<String>,
    },
    /// 不支持分块且大小未知/较大的文件（需单次完整下载）
    Single {
        url: String,
        path: PathBuf,
        abort: Arc<AtomicBool>,
        sha1: Option<String>,
    },
}

pub struct Probe {
    total: Option<usize>,
    ranges: bool,
}

#[derive(Clone, Debug)]
pub struct DownloadTask {
    pub url: String,
    pub path: PathBuf,
    pub sha1: Option<String>,
}

#[derive(Clone)]
pub struct Downloader {
    client: Client,
    threads: usize,
    chunk_size: usize,
    retries: usize,
    progressor: Arc<Progressor>,
    tasks: Vec<DownloadTask>,
}

impl Downloader {
    pub fn new(
        tasks: Vec<DownloadTask>,
        threads: usize,
        chunk_size: usize,
        retries: usize,
    ) -> anyhow::Result<Self> {
        let client = Client::builder()
            .connect_timeout(Duration::from_secs(15))
            .timeout(Duration::from_secs(300))
            .user_agent(concat!("chunk-dl/", env!("CARGO_PKG_VERSION")))
            .build()?;
        Ok(Self {
            client,
            threads: threads.max(1),
            chunk_size: chunk_size.max(1),
            retries,
            progressor: Progressor::new(),
            tasks,
        })
    }

    /// 注册进度回调（前端事件推送 / 终端输出等）。
    pub fn on_progress(self, f: impl Fn(Snapshot) + Send + Sync + 'static) -> Self {
        self.progressor.set_sink(Arc::new(f));
        self
    }

    pub fn progressor(&self) -> &Arc<Progressor> {
        &self.progressor
    }

    pub fn add_task(&mut self, task: DownloadTask) {
        self.tasks.push(task);
    }

    fn small_limit(&self) -> usize {
        self.chunk_size.min(SMALL_FILE_LIMIT)
    }

    /// `Range: bytes=0-0`：206 + Content-Range 说明支持分块且能拿到总大小。
    /// 416 说明是空文件（0 字节无法满足 0-0 的范围）。
    fn probe(&self, task: &DownloadTask) -> anyhow::Result<Probe> {
        let resp = self.client.get(&task.url).header(RANGE, "bytes=0-0").send()?;
        let status = resp.status();
        if status == StatusCode::PARTIAL_CONTENT {
            let total = resp
                .headers()
                .get(CONTENT_RANGE)
                .and_then(|v| v.to_str().ok())
                .and_then(|s| s.rsplit('/').next())
                .and_then(|s| s.trim().parse::<usize>().ok());
            Ok(Probe { total, ranges: total.is_some() })
        } else if status == StatusCode::RANGE_NOT_SATISFIABLE {
            Ok(Probe { total: Some(0), ranges: false })
        } else if status.is_success() {
            Ok(Probe { total: resp.content_length().map(|n| n as usize), ranges: false })
        } else {
            bail!(format!("探测失败: HTTP {status}"))
        }
    }

    /// 并行探测所有文件（文件很多且都很小时，串行探测会成为主要瓶颈）。
    fn probe_all(&self) -> anyhow::Result<Vec<Probe>> {
        let n = self.tasks.len();
        let next = AtomicUsize::new(0);
        let results: Mutex<Vec<Option<anyhow::Result<Probe>>>> =
            Mutex::new((0..n).map(|_| None).collect());

        thread::scope(|s| {
            for _ in 0..self.threads.min(n) {
                s.spawn(|| loop {
                    let i = next.fetch_add(1, Ordering::Relaxed);
                    if i >= n {
                        break;
                    }
                    let r = self.probe(&self.tasks[i]);
                    results.lock().unwrap()[i] = Some(r);
                });
            }
        });

        results
            .into_inner()?
            .into_iter()
            .map(|r| r.expect("probe result missing"))
            .collect()
    }

    /// 阻塞运行；前端场景请放到独立线程里调用。
    pub fn run(&self) -> anyhow::Result<()> {
        self.progressor.set_status(Status::Probing);
        let reporter = self.progressor.spawn_reporter();

        let result = self.run_all();
        match &result {
            Ok(()) => self.progressor.set_status(Status::Done),
            Err(e) => self.progressor.fail(e.to_string()),
        }
        self.progressor.finish();
        reporter.join().ok();
        result
    }

    fn run_all(&self) -> anyhow::Result<()> {
        // 1. 并行探测所有文件大小
        let probes = self.probe_all()?;

        let mut total_bytes = 0;
        let mut all_known = true;
        for info in &probes {
            match info.total {
                Some(t) => total_bytes += t,
                None => all_known = false,
            }
        }

        if all_known {
            self.progressor.set_total(Some(total_bytes));
        }
        self.progressor.set_status(Status::Downloading);

        // 2. 构建全局任务队列
        let mut queue = Vec::new();
        let small_limit = self.small_limit();

        for (task, info) in self.tasks.iter().zip(probes.into_iter()) {
            let abort = Arc::new(AtomicBool::new(false)); // 用于某块失败时中止该文件的其他块

            if let Some(parent) = task.path.parent() {
                std::fs::create_dir_all(parent)?;
            }

            match (info.ranges, info.total) {
                // 小文件（含 0 字节、以及不支持 Range 但大小已知的小文件）
                (_, Some(size)) if size <= small_limit => {
                    queue.push(SubTask::Small {
                        url: task.url.clone(),
                        path: task.path.clone(),
                        size,
                        sha1: task.sha1.clone(),
                    });
                }
                // 大文件：分块
                (true, Some(total)) => {
                    let file = OpenOptions::new()
                        .create(true).write(true).truncate(true)
                        .open(&task.path)?;
                    file.set_len(total as u64)?;
                    let file = Arc::new(file);

                    let nchunks = total.div_ceil(self.chunk_size);
                    // 同一文件的所有分块必须共用一个计数器，
                    // 否则永远等不到 prev == 1，校验不会被触发。
                    let chunk_left = Arc::new(AtomicUsize::new(nchunks));
                    for i in 0..nchunks {
                        let start = i * self.chunk_size;
                        let end = (start + self.chunk_size - 1).min(total - 1);
                        queue.push(SubTask::Chunk {
                            url: task.url.clone(),
                            path: task.path.clone(),
                            file: file.clone(),
                            start,
                            end,
                            abort: abort.clone(),
                            chunk_left: chunk_left.clone(),
                            sha1: task.sha1.clone(),
                        });
                    }
                }
                _ => {
                    queue.push(SubTask::Single {
                        url: task.url.clone(),
                        path: task.path.clone(),
                        abort: abort.clone(),
                        sha1: task.sha1.clone(),
                    });
                }
            }
        }

        // 使用 Mutex 包裹队列，供多线程安全提取（为了从前往后下，我们可以将其翻转后用 pop 取）
        queue.reverse();
        let shared_queue = Arc::new(Mutex::new(queue));

        // 3. 启动固定数量的工作线程池 (严格不超过 self.threads)
        thread::scope(|s| {
            let mut handles = Vec::new();
            for _ in 0..self.threads {
                let queue = shared_queue.clone();
                let dl = self.clone();

                handles.push(s.spawn(move || -> anyhow::Result<()> {
                    loop {
                        // 加锁提取下一个任务
                        let job = {
                            let mut q = queue.lock().unwrap();
                            q.pop()
                        };

                        let Some(job) = job else {
                            break Ok(()); // 队列为空，线程安全退出
                        };

                        match job {
                            SubTask::Chunk { url, path, file, start, end, abort, chunk_left, sha1 } => {
                                if abort.load(Ordering::Relaxed) { continue; }
                                if let Err(e) = dl.download_chunk(&url, &file, start, end, &abort) {
                                    abort.store(true, Ordering::Relaxed);
                                    return Err(e);
                                }
                                let prev = chunk_left.fetch_sub(1, Ordering::SeqCst);
                                if prev == 1 {
                                    file.sync_all()?;
                                    if let Some(sha1) = &sha1 {
                                        if !verify(sha1, &path) {
                                            bail!("file: {} verification failed", path.display());
                                        }
                                    }
                                }
                            }
                            SubTask::Small { url, path, size, sha1 } => {
                                dl.download_small(&url, &path, size)?;
                                if let Some(sha1) = sha1 {
                                    if !verify(&sha1, &path) {
                                        bail!("file: {} verification failed", path.display());
                                    }
                                }
                            }
                            SubTask::Single { url, path, abort, sha1 } => {
                                if abort.load(Ordering::Relaxed) { continue; }
                                if let Err(e) = dl.download_single(&url, &path) {
                                    abort.store(true, Ordering::Relaxed);
                                    return Err(e);
                                }
                                if let Some(sha1) = sha1 {
                                    if !verify(&sha1, &path) {
                                        bail!("file: {} verification failed", path.display());
                                    }
                                }
                            }
                        }
                    }
                }));
            }

            // 4. 等待所有线程执行完毕并收集错误
            let mut first_err: Option<Box<dyn Error + Send + Sync>> = None;
            for h in handles {
                match h.join() {
                    Ok(Ok(())) => {}
                    Ok(Err(e)) => {
                        // 保留最早的真实错误，忽略其他线程因中止而产生的错误
                        if first_err.is_none() && e.to_string() != "已中止" {
                            first_err = Some(Box::from(e));
                        }
                    }
                    Err(_) => {
                        first_err.get_or_insert_with(|| "工作线程 panic".into());
                    }
                }
            }
            if let Some(e) = first_err {
                bail!(e);
            }
            Ok(())
        })
    }

    /// 小文件：整体读入内存 -> 校验长度 -> 写盘。
    /// 因为进度只在成功后一次性上报，失败重试不会造成进度重复累加。
    fn download_small(&self, url: &str, path: &PathBuf, expected: usize) -> anyhow::Result<()> {
        let mut attempt = 0usize;
        loop {
            let res = (|| -> anyhow::Result<usize> {
                let resp = self.client.get(url).send()?.error_for_status()?;
                let data = resp.bytes()?;
                if data.len() != expected {
                    bail!("大小不一致: 期望 {expected}，实际 {}", data.len());
                }
                std::fs::write(path, &data)?;
                Ok(data.len())
            })();

            match res {
                Ok(n) => {
                    self.progressor.add(n);
                    return Ok(());
                }
                Err(e) => {
                    attempt += 1;
                    if attempt > self.retries {
                        bail!("小文件 {url} 重试 {} 次后仍失败: {e}", self.retries);
                    }
                    let backoff = 500u64 << (attempt - 1).min(5);
                    thread::sleep(Duration::from_millis(backoff));
                }
            }
        }
    }

    /// 服务器不支持 Range 且大小未知/较大时的单线程回退。
    fn download_single(&self, url: &str, path: &PathBuf) -> anyhow::Result<()> {
        let mut resp = self.client.get(url).send()?.error_for_status()?;
        if path.exists() {
            std::fs::remove_file(path)?;
        }
        let mut file = File::create(path)?;
        let mut buf = vec![0u8; 64 * 1024];
        loop {
            let n = resp.read(&mut buf)?;
            if n == 0 { break; }
            file.write_all(&buf[..n])?;
            self.progressor.add(n);
        }
        file.sync_all()?;
        Ok(())
    }

    /// 单个块：失败后指数退避重试，并从已写入的位置续传。
    fn download_chunk(&self, url: &str, file: &File, start: usize, end: usize, abort: &AtomicBool) -> anyhow::Result<()> {
        let mut pos = start;
        let mut attempt = 0usize;
        while pos <= end {
            match self.try_range(url, file, &mut pos, end, abort) {
                Ok(()) => {}
                Err(e) => {
                    if abort.load(Ordering::Relaxed) {
                        return Err(e);
                    }
                    attempt += 1;
                    if attempt > self.retries {
                        bail!(format!(
                            "块 {start}-{end} 重试 {} 次后仍失败: {e}",
                            self.retries));
                    }
                    let backoff = 500u64 << (attempt - 1).min(5);
                    thread::sleep(Duration::from_millis(backoff));
                }
            }
        }
        Ok(())
    }

    fn try_range(&self, url: &str, file: &File, pos: &mut usize, end: usize, abort: &AtomicBool) -> anyhow::Result<()> {
        let mut resp = self
            .client
            .get(url)
            .header(RANGE, format!("bytes={}-{}", *pos, end))
            .send()?;
        if resp.status() != StatusCode::PARTIAL_CONTENT {
            bail!(format!("期望 206，实际 {}", resp.status()));
        }
        let mut buf = vec![0u8; 64 * 1024];
        while *pos <= end {
            if abort.load(Ordering::Relaxed) {
                bail!("已中止");
            }
            let want = (end - *pos + 1).min(buf.len());
            let n = resp.read(&mut buf[..want])?;
            if n == 0 {
                bail!("连接提前结束");
            }
            write_at(file, &buf[..n], *pos)?;
            *pos += n;
            self.progressor.add(n);
        }
        Ok(())
    }
}

#[cfg(unix)]
fn write_at(f: &File, buf: &[u8], off: usize) -> std::io::Result<()> {
    use std::os::unix::fs::FileExt;
    f.write_all_at(buf, off as u64)
}

#[cfg(windows)]
fn write_at(f: &File, mut buf: &[u8], mut off: usize) -> std::io::Result<()> {
    use std::os::windows::fs::FileExt;
    while !buf.is_empty() {
        let n = f.seek_write(buf, off as u64)?;
        buf = &buf[n..];
        off += n;
    }
    Ok(())
}