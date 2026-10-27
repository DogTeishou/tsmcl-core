use std::sync::{Arc, Mutex};
use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::thread::JoinHandle;
use std::time::Duration;
use tokio::time::Instant;
use serde::Serialize;

pub type Sink = Arc<dyn Fn(Snapshot) + Send + Sync>;

#[derive(Clone, Debug, Serialize)]
pub enum Status {
    Idle,
    Probing,
    Downloading,
    Done,
    Failed,
}

#[derive(Clone, Debug, Serialize)]
pub struct Snapshot {
    pub status: Status,
    pub downloaded: usize,
    pub total: Option<usize>,
    pub percent: Option<f64>,
    pub speed_bps: f64,
    pub eta_secs: Option<f64>,
    pub elapsed_secs: f64,
    pub error: Option<String>,
}

pub struct Progressor {
    downloaded: AtomicUsize,
    total: AtomicUsize,
    finished: AtomicBool,
    started: Instant,
    state: Mutex<(Status, Option<String>)>,
    sink: Mutex<Option<Sink>>,
}

impl Progressor {
    pub fn new() -> Arc<Self> {
        Arc::new(Self {
            downloaded: AtomicUsize::new(0),
            total: AtomicUsize::new(0),
            finished: AtomicBool::new(false),
            started: Instant::now(),
            state: Mutex::new((Status::Idle, None)),
            sink: Mutex::new(None),
        })
    }

    pub fn set_sink(&self, sink: Sink) -> &Self {
        *self.sink.lock().unwrap() = Some(sink);
        self
    }

    pub fn set_total(&self, total: Option<usize>) {
        self.total.store(total.unwrap_or(0), Ordering::Relaxed);
    }

    pub fn set_status(&self, status: Status) {
        self.state.lock().unwrap().0 = status;
    }

    pub fn fail(&self, msg: String) {
        *self.state.lock().unwrap() = (Status::Failed, Some(msg));
    }

    pub fn add(&self, n: usize) {
        self.downloaded.fetch_add(n, Ordering::Relaxed);
    }

    pub fn downloaded(&self) -> usize {
        self.downloaded.load(Ordering::Relaxed)
    }

    pub fn finish(&self) -> &Self {
        self.finished.store(true, Ordering::Relaxed);
        self
    }

    /// cancel current snapshot
    pub fn snapshot(&self) -> Snapshot {
        self.build(None)
    }

    fn build(&self, speed: Option<f64>) -> Snapshot {
        let downloaded = self.downloaded();
        let total = match self.total.load(Ordering::Relaxed) {
            0 => None,
            t => Some(t),
        };
        let elapsed_secs = self.started.elapsed().as_secs_f64();
        let speed_bps = speed.unwrap_or_else(|| downloaded as f64 / elapsed_secs.max(0.001));
        let percent = total.map(|t| (downloaded as f64 / t as f64 * 100.0).min(100.0));
        let eta_secs = total.and_then(|t| {
            if speed_bps > 1.0 {
                Some((t - downloaded.min(t)) as f64 / speed_bps)
            } else {
                None
            }
        });
        let (status, error) = self.state.lock().unwrap().clone();
        Snapshot { status, downloaded, total, percent, speed_bps, eta_secs, elapsed_secs, error }
    }

    fn emit(&self, snap: Snapshot) {
        let sink = self.sink.lock().unwrap().clone();
        if let Some(f) = sink {
            f(snap);
        }
    }

    pub fn spawn_reporter(self: &Arc<Self>) -> JoinHandle<()> {
        let me = Arc::clone(self);
        std::thread::spawn(move || {
            let mut last_t = Instant::now();
            let mut last_b = me.downloaded();
            let mut ema: Option<f64> = None;
            loop {
                let finished = me.finished.load(Ordering::Relaxed);
                let now = Instant::now();
                let got = me.downloaded();
                let dt = now.duration_since(last_t).as_secs_f64();
                if dt > 0.0 {
                    let inst = (got - last_b) as f64 / dt;
                    ema = Some(match ema {
                        Some(e) => 0.3 * inst + 0.7 * e,
                        None => inst,
                    });
                    last_t = now;
                    last_b = got;
                }
                // 最后一帧用整体平均速度
                me.emit(me.build(if finished { None } else { ema }));
                if finished {
                    break;
                }
                std::thread::sleep(Duration::from_millis(200));
            }
        })
    }
}

pub fn human(n: f64) -> String {
    const U: [&str; 5] = ["B", "KiB", "MiB", "GiB", "TiB"];
    let (mut v, mut i) = (n, 0);
    while v >= 1024.0 && i < U.len() - 1 {
        v /= 1024.0;
        i += 1;
    }
    format!("{v:.2} {}", U[i])
}

pub fn terminal_sink(s: Snapshot) {
    match (s.total, s.percent) {
        (Some(t), Some(p)) => eprint!(
            "\r{:>6.2}%  {} / {}  {}/s  ETA {:>4.0}s   ",
            p,
            human(s.downloaded as f64),
            human(t as f64),
            human(s.speed_bps),
            s.eta_secs.unwrap_or(0.0)
        ),
        _ => eprint!("\r{}  {}/s   ", human(s.downloaded as f64), human(s.speed_bps)),
    }
    if matches!(s.status, Status::Done | Status::Failed) {
        eprintln!();
    }
}