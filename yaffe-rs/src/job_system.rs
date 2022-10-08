use std::thread;
use std::cell::RefCell;
use std::sync::Arc;
use crate::net_api::*;
use crate::modals::{ListModal, display_modal, ModalSize, on_platform_found_close};
use crate::logger::*;

pub type JobQueue = spmc::Sender<JobType>;
pub type ThreadSafeJobQueue = Arc<std::sync::Mutex<RefCell<JobQueue>>>;

//This is used to pass a raw pointer to the assetslot between threads
//Use very rarely when mutability and lifetimes cause issues
//Passing YaffeState around is currently safe due to internal structures being threadsafe
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

    (tx, notify_rx)
}

fn poll_pending_jobs(queue: spmc::Receiver<JobType>, notify: std::sync::mpsc::Sender<u8>) {
    while let Ok(msg) = queue.recv() {
        match msg {
            JobType::LoadImage((path, slot)) => crate::assets::load_image_async(path, slot),
    
            JobType::DownloadUrl((t, url, path)) => {
                crate::logger::info!("Downloading file from {}", url.to_str().unwrap());

                match crate::net_api::send_request_no_parms(t, url.to_str().unwrap()) {
                    Err(e) => crate::logger::error!("{:?}", e),
                    Ok(bytes) => {
                        //Download and write file to disk
                        let file = bytes.bytes().unwrap();
                        std::fs::write(path, file).log_and_panic();
                    }
                }
            }

            JobType::SearchPlatform((state, name, path, args)) => {
                let state = state.get_inner::<crate::YaffeState>();
                if let Some(result) = search_platform(&name).display_failure("Unable to send message for platform search", state) {

                    if let Some(plat) = result.get_exact() {
                        let plat = crate::database::PlatformData::new(plat, path.clone(), args.clone());
                        crate::platform::insert_platform(state, &plat);

                    } else if result.count > 0 {
                        let mut content: ListModal<crate::database::PlatformData> = ListModal::new(Some(format!("Found {} results for '{}'", result.count, &name)));
                        for i in result.results {
                            content.add_item(crate::database::PlatformData::new(&i, path.clone(), args.clone()));
                        }

                        display_modal(state, "Select Platform", None, Box::new(content), ModalSize::Half, Some(on_platform_found_close));
                    }
                }
            }

            JobType::SearchGame((state, exe, name, plat_id)) => {
                let state = state.get_inner::<crate::YaffeState>();
                if let Some(result) = search_game(&name, plat_id).display_failure("Unable to send message for game search", state) {

                    if let Some(game) = result.get_exact() {
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
    /// Loads an image synchronously
    /// Should only be called through the asset system
    /// We copy the AssetPathType from the slot so 
    /// the locks on the slot are shorter
    LoadImage((crate::assets::AssetPathType, RawDataPointer)),

    /// Downloads the file at a given url and writes it to the file system
    DownloadUrl((crate::net_api::Authentication, std::path::PathBuf, std::path::PathBuf)),

    /// Searches TheGamesDb for a given platform
    SearchPlatform((RawDataPointer, String, String, String)),

    /// Searches TheGamesDb for a given game
    SearchGame((RawDataPointer, String, String, i64)),
}