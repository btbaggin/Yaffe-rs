use rand::Rng;
use std::cell::RefCell;
use std::path::PathBuf;
use std::sync::{
    mpsc::{Receiver, Sender},
    Arc, Mutex,
};
use winit::window::WindowId;

use crate::assets::AssetKey;
use crate::logger::*;
use crate::scraper::*;
use crate::windowing::get_current_window;

pub type JobQueue = spmc::Sender<(Option<WindowId>, Job)>;
pub type JobResults = Receiver<(Option<WindowId>, JobResult)>;

#[derive(Clone)]
pub struct ThreadSafeJobQueue(Arc<Mutex<RefCell<JobQueue>>>);
impl ThreadSafeJobQueue {
    pub fn new(queue: JobQueue) -> ThreadSafeJobQueue { ThreadSafeJobQueue(Arc::new(Mutex::new(RefCell::new(queue)))) }

    pub fn start_job(&self, job: Job) {
        let lock = self.0.lock().log_and_panic();
        let mut queue = lock.borrow_mut();
        queue.send((Some(get_current_window()), job)).unwrap();
    }

    pub fn start_unassociated_job(&self, job: Job) {
        let lock = self.0.lock().log_and_panic();
        let mut queue = lock.borrow_mut();
        queue.send((None, job)).unwrap();
    }
}

/// Starts a single producer multiple consumer job threading system
/// Jobs can be sent to this system using the returned JobQueue
pub fn start_job_system() -> (ThreadSafeJobQueue, JobResults) {
    const NUM_THREADS: u32 = 8;

    let (tx, rx) = spmc::channel();
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();
    for _ in 0..NUM_THREADS {
        let rx = rx.clone();
        let notify_tx = notify_tx.clone();
        std::thread::spawn(move || poll_pending_jobs(rx, notify_tx));
    }

    (ThreadSafeJobQueue::new(tx), notify_rx)
}

fn poll_pending_jobs(queue: spmc::Receiver<(Option<WindowId>, Job)>, notify: Sender<(Option<WindowId>, JobResult)>) {
    let send_reply = |window_id, result| {
        notify.send((window_id, result)).log("Unable to notify main loop about finished job");
    };

    while let Ok((window_id, msg)) = queue.recv() {
        crate::logger::trace!("Processing job {msg:?}");
        match msg {
            Job::LoadImage { key, file } => {
                if let Some((data, dimensions)) = crate::assets::load_image_async(&key, file) {
                    send_reply(window_id, JobResult::LoadImage { data, dimensions, key });
                }
            }

            Job::DownloadUrl { url, file_path } => crate::scraper::download_file(url, file_path),

            Job::SearchPlatform { name, path, args } => {
                let result = search_platform(&name, path, args);
                send_reply(window_id, JobResult::SearchPlatform(result));
            },

            Job::SearchGame { exe, name, platform } => {
                let result = search_game(&name, exe, platform);
                send_reply(window_id, JobResult::SearchGame(result));
            },

            Job::CheckUpdates => {
                let applied = crate::scraper::check_for_updates().log("Error checking for updates");
                send_reply(window_id, JobResult::CheckUpdates(applied))
            }
        }
    }
}

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
        name: String,
        path: String,
        args: String,
    },

    /// Searches TheGamesDb for a given game
    SearchGame {
        exe: String,
        name: String,
        platform: i64,
    },

    CheckUpdates,
}

pub enum JobResult {
    LoadImage { data: Vec<u8>, dimensions: (u32, u32), key: AssetKey },
    SearchPlatform(ServiceResult<ServiceResponse<PlatformScrapeResult>>),
    SearchGame(ServiceResult<ServiceResponse<GameScrapeResult>>),
    CheckUpdates(bool),
}
