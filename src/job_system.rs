use std::thread;
use crate::server::*;
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
pub fn start_job_system() -> JobQueue {
    const NUM_THREADS: u32 = 4;

    let (tx, rx) = spmc::channel();
    for _ in 0..NUM_THREADS {
        let rx = rx.clone();
        thread::spawn(move || {
            poll_pending_jobs(rx)
        });
    }

    JobQueue { queue: tx, set: HashSet::new() }
}

fn poll_pending_jobs(queue: spmc::Receiver<JobType>) {
    loop {
        let msg = queue.recv().unwrap();
        match msg {
            JobType::LoadImage(slot) => crate::assets::load_image_async(slot),
    
            JobType::DownloadUrl((url, path)) => {
                let url = std::path::Path::new("https://cdn.thegamesdb.net/images/medium/").join(url);
                let image = reqwest::blocking::get(url.to_str().unwrap()).unwrap().bytes().unwrap();
                std::fs::write(path, image).log_if_fail();
            }

            JobType::SearchPlatform((state, name, path, args, rom)) => {
                let message = MessageTypes::PlatformInfo(&name);
                let state = state.get_inner::<crate::YaffeState>();
                if let Some(buffer) = send_message(message).display_failure("Unable to send message for platform search", state) {
                    let result: ServiceResponse<PlatformInfo> = serde_json::from_slice(&buffer).log_if_fail();

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
                let message = MessageTypes::GameInfo((plat_id, &name));
                let state = state.get_inner::<crate::YaffeState>();
                if let Some(buffer) = send_message(message).display_failure("Unable to send message for game search", state) {
                    let result: ServiceResponse<GameInfo> = serde_json::from_slice(&buffer).log_if_fail();

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
    }
}

pub enum JobType {
    LoadImage(RawDataPointer),
    DownloadUrl((String, String)),
    SearchPlatform((RawDataPointer, String, String, String, String)),
    SearchGame((RawDataPointer, String, String, i64))
}

/// Does unsafe nonesense to get a mutable reference to the job queue that is shared
/// by all widgets. This is currently safe because widget processing is single threaded
pub fn get_queue_mut<'a>(queue: &std::sync::Arc<JobQueue>) -> &'a mut JobQueue {
    unsafe { &mut *(std::sync::Arc::as_ptr(queue) as *mut JobQueue) }
}