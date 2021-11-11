use std::thread;
use crate::game_api::*;
use crate::modals::{ListModal, display_modal, ModalSize, on_platform_found_close};
use crate::logger::*;
use std::collections::HashSet;

//This is used to pass a raw pointer to the assetslot between threads
//Since PietImage isnt marked Send we just pass the raw pointer, we don't touch the image off the main thread
#[derive(Clone, Copy)]
pub struct RawDataPointer(*mut u8);
unsafe impl std::marker::Send for RawDataPointer {}
impl RawDataPointer {
    pub fn new<T>(item: &mut T) -> RawDataPointer {
        unsafe {
            let layout = std::alloc::Layout::new::<usize>();
            let slot_ptr = std::alloc::alloc(layout);
            *(slot_ptr as *mut &T) = item;
            RawDataPointer(slot_ptr)
        }
    }
    pub fn get_inner<'a, T>(&self) -> &'a mut T {
        unsafe { &mut *(self.0 as *mut &mut T) }
    }
}

pub struct JobQueue {
    queue: spmc::Sender<JobType>,
    set: HashSet<String>,
} 
impl JobQueue {
    /// Returns if `send_with_key` has been called before with this key
    pub fn already_sent(&self, key: String) -> bool {
        self.set.get(&key).is_some()
    }

    /// Sends a message to the job system for asynchronous processing
    /// Each new message type needs custom handling
    pub fn send(&mut self, job: JobType) {
        self.queue.send(job).unwrap()
    }

    /// Same as `send` but allows `already_sent` to check if its already been used
    pub fn send_with_key(&mut self, key: String, job: JobType) {
        self.set.insert(key);
        self.send(job);
    }
}

/// Starts a single producer multiple consumer job threading system
/// Jobs can be sent to this system using the returned JobQueue
pub fn start_job_system() -> (JobQueue, std::sync::mpsc::Receiver<u8>) {
    const NUM_THREADS: u32 = 8;

    let (tx, rx) = spmc::channel();
    let (notify_tx, notify_rx) = std::sync::mpsc::channel();
    for _ in 0..NUM_THREADS {
        let rx = rx.clone();
        let notify_tx = notify_tx.clone();
        thread::spawn(move || {
            poll_pending_jobs(rx, notify_tx)
        });
    }

    (JobQueue { queue: tx, set: HashSet::new() }, notify_rx)
}

fn poll_pending_jobs(queue: spmc::Receiver<JobType>, notify: std::sync::mpsc::Sender<u8>) {
    loop {
        let msg = queue.recv().log_and_panic();
        match msg {
            JobType::LoadImage(slot) => crate::assets::load_image_async(slot),
    
            JobType::DownloadUrl((url, path)) => {
                let url = std::path::Path::new("https://cdn.thegamesdb.net/images/medium/").join(url);

                //Download and write file to disk
                match reqwest::blocking::get(url.to_str().unwrap()) {
                    Err(e) => crate::logger::log_entry(LogTypes::Error, e),
                    Ok(bytes) => {
                        let image = bytes.bytes().unwrap();
                        std::fs::write(path, image).log_and_panic();
                    }
                } 
            }

            JobType::SearchPlatform((state, name, path, args, rom)) => {
                let state = state.get_inner::<crate::YaffeState>();
                if let Some(result) = search_platform(&name).display_failure("Unable to send message for platform search", state) {

                    if result.exact {
                        let plat = &result.results[0];
                        let plat = crate::database::PlatformData::new(plat, path.clone(), args.clone(), rom.clone());
                        crate::platform::insert_platform(state, &plat);

                    } else if result.count > 0 {
                        let mut content: ListModal<crate::database::PlatformData> = ListModal::new(Some(format!("Found {} results for '{}'", result.count, &name)));
                        for i in result.results {
                            content.add_item(crate::database::PlatformData::new(&i, path.clone(), args.clone(), rom.clone()));
                        }

                        display_modal(state, "Select Platform", None, Box::new(content), ModalSize::Half, Some(on_platform_found_close));
                    }
                }
            }

            JobType::SearchGame((state, exe, name, plat_id)) => {
                let state = state.get_inner::<crate::YaffeState>();
                if let Some(result) = search_game(&name, plat_id).display_failure("Unable to send message for game search", state) {

                    if result.exact {
                        let game = &result.results[0];
                        let data = crate::database::GameData::new(game, exe.clone(), plat_id);
                        crate::platform::insert_game(state, &data);

                    } else if result.count > 0 {
                        let mut content: ListModal<crate::database::GameData> = ListModal::new(Some(format!("Found {} results for '{}'", result.count, &name)));
                        for i in result.results {
                            content.add_item(crate::database::GameData::new(&i, exe.clone(), plat_id));
                        }

                        display_modal(state, "Select Game", None, Box::new(content), ModalSize::Half, Some(crate::modals::on_game_found_close));
                    }
                }
            }
        }

        notify.send(0).log("Unable to notify main loop about finished job");
    }
}

pub enum JobType {
    LoadImage(RawDataPointer),
    DownloadUrl((String, String)),
    SearchPlatform((RawDataPointer, String, String, String, String)),
    SearchGame((RawDataPointer, String, String, i64)),
}