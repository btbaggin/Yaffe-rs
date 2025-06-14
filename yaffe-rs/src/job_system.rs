use rand::Rng;
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use std::thread;

use crate::assets::AssetKey;
use crate::logger::*;
use crate::scraper::*;

pub type JobQueue = spmc::Sender<Job>;
pub type JobResults = std::sync::mpsc::Receiver<JobResult>;
pub type ThreadSafeJobQueue = Arc<Mutex<RefCell<JobQueue>>>;

/// Starts a single producer multiple consumer job threading system
/// Jobs can be sent to this system using the returned JobQueue
pub fn start_job_system() -> (ThreadSafeJobQueue, std::sync::mpsc::Receiver<JobResult>) {
    const NUM_THREADS: u32 = 8;

    let (tx, rx) = spmc::channel();
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();
    for _ in 0..NUM_THREADS {
        let rx = rx.clone();
        let notify_tx = notify_tx.clone();
        thread::spawn(move || poll_pending_jobs(rx, notify_tx));
    }

    (Arc::new(Mutex::new(RefCell::new(tx))), notify_rx)
}

fn poll_pending_jobs(queue: spmc::Receiver<Job>, notify: std::sync::mpsc::Sender<JobResult>) {
    let send_reply = |result| {
        notify.send(result).log("Unable to notify main loop about finished job");
    };

    while let Ok(msg) = queue.recv() {
        crate::logger::trace!("Processing job {msg:?}");
        match msg {
            Job::LoadImage { key, file } => {
                if let Some((data, dimensions)) = crate::assets::load_image_async(&key, file) {
                    send_reply(JobResult::LoadImage { data, dimensions, key });
                }
            }

            Job::DownloadUrl { url, file_path } => crate::scraper::download_file(url, file_path),

            Job::SearchPlatform { id, name, path, args } => match search_platform(id, &name, path, args) {
                Ok(result) => send_reply(JobResult::SearchPlatform(result)),
                Err(e) => error!("Error occured while searching platforms {e:?}"),
            },

            Job::SearchGame { id, exe, name, platform } => match search_game(id, &name, exe, platform) {
                Ok(result) => send_reply(JobResult::SearchGame(result)),
                Err(e) => error!("Error occured while searching games {e:?}"),
            },

            Job::CheckUpdates => {
                let applied = crate::scraper::check_for_updates().log("Error checking for updates");
                send_reply(JobResult::CheckUpdates(applied))
            }
        }
    }
}

pub fn generate_job_id() -> u64 { rand::rng().random::<u64>() }

#[derive(Debug)]
pub enum Job {
    /// Loads an image synchronously
    LoadImage {
        key: AssetKey,
        file: PathBuf,
    },

    /// Downloads the file at a given url and writes it to the file system
    DownloadUrl {
        url: std::path::PathBuf,
        file_path: std::path::PathBuf,
    },

    /// Searches TheGamesDb for a given platform
    SearchPlatform {
        id: u64,
        name: String,
        path: String,
        args: String,
    },

    /// Searches TheGamesDb for a given game
    SearchGame {
        id: u64,
        exe: String,
        name: String,
        platform: i64,
    },

    CheckUpdates,
}

#[derive(Clone)]
pub enum JobResult {
    None,
    LoadImage { data: Vec<u8>, dimensions: (u32, u32), key: AssetKey },
    SearchPlatform(ServiceResponse<PlatformScrapeResult>),
    SearchGame(ServiceResponse<GameScrapeResult>),
    CheckUpdates(bool),
}

pub fn process_results<F, T>(results: &mut Vec<JobResult>, should_process: F, mut process: T)
where
    F: Fn(&JobResult) -> bool,
    T: FnMut(JobResult),
{
    for r in results.iter_mut() {
        if should_process(r) {
            let result = std::mem::replace(r, JobResult::None);
            process(result);
        }
    }

    results.retain(|r| !matches!(r, JobResult::None))
}
