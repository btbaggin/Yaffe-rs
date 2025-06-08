use crate::graphics::Graphics;
use crate::logger::PanicLogEntry;
use crate::pooled_cache::PooledCache;
use speedy2d::{font::*, image::*};
use std::path::{Path, PathBuf};
use std::rc::Rc;
use std::sync::atomic::{AtomicU8, Ordering};
use std::time::Instant;

mod atlas;
mod font;
mod image;
use self::image::preload_image;
pub use self::image::{load_image_async, Images, YaffeTexture};
use atlas::load_texture_atlas;
pub use font::Fonts;

const ASSET_STATE_UNLOADED: u8 = 0;
const ASSET_STATE_PENDING: u8 = 1;
const ASSET_STATE_LOADED: u8 = 2;

#[derive(Debug, Clone, Hash, PartialEq, Eq)]
pub enum AssetKey {
    File(PathBuf),
    Url(PathBuf),
    Static(AssetTypes),
}
impl AssetKey {
    fn get_path(self) -> PathBuf {
        match self {
            AssetKey::File(p) => p,
            AssetKey::Url(p) => p,
            AssetKey::Static(_) => unimplemented!(),
        }
    }
    fn image(image: Images) -> AssetKey { AssetKey::Static(AssetTypes::Image(image)) }
}

#[derive(Debug, PartialEq, Eq, Hash, Clone, Copy)]
pub enum AssetTypes {
    Image(Images),
    Font(Fonts),
}

#[derive(Clone)]
pub enum AssetData {
    Image(YaffeTexture),
    Font(Font),
    Raw((Vec<u8>, (u32, u32))),
    None,
}

pub struct AssetSlot {
    state: AtomicU8,
    path: PathBuf,
    data: AssetData,
    data_length: usize,
    last_request: Instant,
}
impl AssetSlot {
    pub fn new(path: PathBuf) -> AssetSlot {
        AssetSlot {
            state: AtomicU8::new(ASSET_STATE_UNLOADED),
            path,
            data: AssetData::None,
            data_length: 0,
            last_request: Instant::now(),
        }
    }

    pub fn preloaded(path: &str, image: YaffeTexture) -> AssetSlot {
        let size = image.size();
        AssetSlot {
            state: AtomicU8::new(ASSET_STATE_LOADED),
            path: Path::new(path).to_path_buf(),
            data: AssetData::Image(image),
            data_length: (size.x * size.y * 4.) as usize,
            last_request: Instant::now(),
        }
    }

    pub fn font(path: &str) -> AssetSlot {
        let data = std::fs::read(path).log_and_panic();
        let font = speedy2d::font::Font::new(&data).log_and_panic();

        AssetSlot {
            state: AtomicU8::new(ASSET_STATE_LOADED),
            path: Path::new(path).to_path_buf(),
            data: AssetData::Font(font),
            data_length: data.len(),
            last_request: Instant::now(),
        }
    }

    pub fn set_data(&mut self, data: Vec<u8>, dimensions: (u32, u32)) {
        self.data_length = data.len();
        self.data = AssetData::Raw((data, dimensions));
        self.state.swap(ASSET_STATE_LOADED, Ordering::AcqRel);
    }
}

pub fn preload_assets(graphics: &mut Graphics) {
    //TODO this sucks
    let g = unsafe { &mut *graphics.graphics_ptr };
    let mut map = graphics.asset_cache.borrow_mut();
    if !map.exists(&AssetKey::image(Images::Error)) {
        let data =
            g.create_image_from_file_path(None, ImageSmoothingMode::Linear, "./Assets/packed.png").log_and_panic();
        let image = Rc::new(data);

        load_texture_atlas(&mut map, image, "./Assets/atlas.tex", "./Assets/packed.png", |image| match image {
            "error.png" => Images::Error,
            "question.png" => Images::Question,
            "arrow_up.png" => Images::ArrowUp,
            "arrow_down.png" => Images::ArrowDown,
            "button_a.png" => Images::ButtonA,
            "button_b.png" => Images::ButtonB,
            "button_x.png" => Images::ButtonX,
            "button_y.png" => Images::ButtonY,
            "apps.png" => Images::App,
            "emulator.png" => Images::Emulator,
            "recents.png" => Images::Recent,
            "speaker.png" => Images::Speaker,
            "settings.png" => Images::Settings,
            "everyone.png" => Images::ErsbEveryone,
            "everyone10.png" => Images::ErsbEveryone10,
            "teen.png" => Images::ErsbTeen,
            "mature.png" => Images::ErsbMature,
            "adults.png" => Images::ErsbAdultOnly,
            _ => panic!("Unknown image found in texture atlas"),
        });
    }

    if map.get_mut(&AssetKey::image(Images::Placeholder)).is_none() {
        preload_image(g, "./Assets/placeholder.jpg", Images::Placeholder, &mut map);
    }

    if !map.exists(&AssetKey::image(Images::Background)) {
        map.insert(
            AssetKey::image(Images::Background),
            AssetSlot::new(Path::new(r"./Assets/background.jpg").to_path_buf()),
        );
    }

    if !map.exists(&AssetKey::Static(AssetTypes::Font(Fonts::Regular))) {
        map.insert(AssetKey::Static(AssetTypes::Font(Fonts::Regular)), AssetSlot::font("./Assets/Roboto-Regular.ttf"));
    }
}

pub fn get_asset_slot<'a>(map: &'a mut PooledCache<32, AssetKey, AssetSlot>, asset: &AssetKey) -> &'a mut AssetSlot {
    if !map.exists(asset) {
        map.insert(asset.clone(), AssetSlot::new(asset.clone().get_path()));
    }
    map.get_mut(asset).log_message_and_panic("Invalid asset slot request")
}
// pub fn get_asset_slot<'a>(graphics: &'a Graphics, asset: &'a AssetKey) -> &'a mut AssetSlot {
//     let mut map = graphics.asset_cache.borrow_mut();
//     if !map.exists(asset) {
//         map.insert(asset.clone(), AssetSlot::new(asset.clone().get_path()));
//     }
//     map.get_mut(asset).log_message_and_panic("Invalid asset slot request")
// }

fn asset_path_is_valid(asset: &AssetKey, path: &Path) -> bool {
    match asset {
        AssetKey::File(_) | AssetKey::Static(_) => path.exists(),
        AssetKey::Url(_) => true,
    }
}

pub fn get_asset_path(platform: &str, name: &str) -> PathBuf {
    let platform = Path::new("./Assets").join(crate::os::sanitize_file(platform));
    let name = crate::os::sanitize_file(name);
    let name = format!("{name}.jpg");
    platform.join(name)
}

pub fn ensure_asset_loaded<'a>(
    queue: crate::job_system::ThreadSafeJobQueue,
    map: &'a mut PooledCache<32, AssetKey, AssetSlot>,
    asset: &AssetKey,
) -> Option<&'a mut AssetSlot> {
    let slot = get_asset_slot(map, asset);

    if slot.state.load(Ordering::Acquire) == ASSET_STATE_UNLOADED && asset_path_is_valid(asset, &slot.path) {
        if let Ok(ASSET_STATE_UNLOADED) =
            slot.state.compare_exchange(ASSET_STATE_UNLOADED, ASSET_STATE_PENDING, Ordering::Acquire, Ordering::Relaxed)
        {
            let queue = queue.as_ref();
            let lock = queue.lock().log_and_panic();
            let mut queue = lock.borrow_mut();

            queue.send(crate::Job::LoadImage { key: asset.clone(), file: slot.path.clone() }).unwrap();
            return None;
        }
    }
    if slot.state.load(Ordering::Acquire) == ASSET_STATE_LOADED {
        Some(slot)
    } else {
        None
    }
}

pub fn clear_old_cache(graphics: &mut Graphics, cache_size: usize) {
    let mut map = graphics.asset_cache.borrow_mut();

    let mut total_memory = 0;
    let mut last_used_index: Option<AssetKey> = None;
    let mut last_request = Instant::now();
    for key in map.keys() {
        if let AssetKey::Static(_) = key {
            // Static assets should not be released
        } else {
            let slot = map.get(key).unwrap();
            if slot.state.load(Ordering::Acquire) == ASSET_STATE_LOADED {
                total_memory += slot.data_length;

                //Find oldest asset
                if slot.last_request.elapsed().as_secs() > 30 && slot.last_request < last_request {
                    last_request = slot.last_request;
                    last_used_index = Some(key.clone());
                }
            }
        }
    }

    //Remove oldest asset if we are over our memory threshold
    //This will happen once per frame until we are under the threshold
    if total_memory > 1024 * 1024 * cache_size {
        if let Some(index) = last_used_index {
            let slot = map.get_mut(&index).unwrap();
            slot.data = AssetData::None;
            slot.state.store(ASSET_STATE_UNLOADED, Ordering::Release);
            crate::logger::info!("Releasing file at {} due to memory pressure", slot.path.display());
        }
    }
}
