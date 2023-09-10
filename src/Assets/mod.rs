use std::sync::atomic::{AtomicU8, Ordering};
use std::rc::Rc;
use std::cell::RefCell;
use crate::logger::PanicLogEntry;
use speedy2d::{Graphics2D, font::*, image::*};
use std::time::Instant;
use crate::pooled_cache::PooledCache;

mod font;
mod image;
mod atlas;
pub use font::{request_font, Fonts};
pub use self::image::{request_asset_image, request_image, load_image_async, Images, YaffeTexture};
pub use yaffe_plugin::PathType as AssetPathType;
use self::image::preload_image;
use atlas::load_texture_atlas;

const ASSET_STATE_UNLOADED: u8 = 0;
const ASSET_STATE_PENDING: u8 = 1;
const ASSET_STATE_LOADED: u8 = 2;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum AssetTypes {
    Image(Images),
    Font(Fonts),
}


pub enum AssetData {
    Image(YaffeTexture),
    Font(Font),
    Raw((Vec<u8>, (u32, u32))),
    None,
}

pub struct AssetSlot {
    state: AtomicU8,
    path: AssetPathType,
    data: AssetData,
    data_length: usize,
    last_request: Instant,
}
impl AssetSlot {
    pub fn new(path: AssetPathType) -> AssetSlot {
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
            path: AssetPathType::File(std::path::Path::new(path).to_path_buf()),
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
            path: AssetPathType::File(std::path::Path::new(path).to_path_buf()),
            data: AssetData::Font(font),
            data_length: data.len(),
            last_request: Instant::now(),
        }
    }
}


//Stores static assets (something from AssetTypes)
static mut STATIC_ASSET_MAP: Option<PooledCache<32, AssetTypes, AssetSlot>> = None;

//Stores dyanmic assets (something loaded from a path)
static mut FILE_ASSET_MAP: Option<PooledCache<32, String, RefCell<AssetSlot>>> = None;

pub fn initialize_asset_cache() {
    let mut map = PooledCache::new();
    map.insert(AssetTypes::Image(Images::Background), AssetSlot::new(AssetPathType::File(std::path::Path::new(r"./Assets/background.jpg").to_path_buf())));

    map.insert(AssetTypes::Font(Fonts::Regular), AssetSlot::font("./Assets/Roboto-Regular.ttf"));
    
    unsafe { STATIC_ASSET_MAP = Some(map); }
    unsafe { FILE_ASSET_MAP = Some(PooledCache::new()); }
}

pub fn preload_assets(graphics: &mut Graphics2D) {
    let map = unsafe { STATIC_ASSET_MAP.as_mut().unwrap() };
    if map.get_mut(&AssetTypes::Image(Images::Error)).is_none() {
        let data = graphics.create_image_from_file_path(None, ImageSmoothingMode::Linear,"./Assets/packed.png").log_and_panic();
        let image = Rc::new(data);

        load_texture_atlas(map, image, "./Assets/atlas.tex", "./Assets/packed.png", |image| {
            match image {
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
            }
        }); 
    }

    if map.get_mut(&AssetTypes::Image(Images::Placeholder)).is_none() {
        preload_image(graphics, "./Assets/placeholder.jpg", Images::Placeholder, map);
    }
}

fn get_slot_mut(t: AssetTypes) -> &'static mut AssetSlot {
    unsafe { STATIC_ASSET_MAP.as_mut().unwrap().get_mut(&t).log_message_and_panic("Invalid asset slot request") }
}

fn asset_path_is_valid(path: &AssetPathType) -> bool {
    match path {
        AssetPathType::File(p) => std::path::Path::new(&p).exists(),
        AssetPathType::Url(_) => true,
    }
}

pub fn get_asset_path(platform: &str, name: &str) -> std::path::PathBuf {
    use std::path::Path;

    let platform = Path::new("./Assets").join(crate::os::sanitize_file(platform));
    let name = crate::os::sanitize_file(name);
    let name = format!("{name}.jpg");
    platform.join(name)
}

pub fn get_cached_file<'a>(file: &AssetPathType) -> &'a RefCell<AssetSlot> {
    //This acts as a cache of exe images
    //If our list ever reloads or we reqeust the same image (recent vs emulator)
    //We will grab the cached image so we dont need to reload the image data
    let map = unsafe { FILE_ASSET_MAP.as_mut().unwrap() };
    let key = match file {
        AssetPathType::File(path) => path,
        AssetPathType::Url(url) => url,
    };
    let key = key.to_string_lossy().to_string();
    map.insert(key.clone(), RefCell::new(AssetSlot::new(file.clone())));
    map.get_mut(&key).unwrap()
}

pub fn clear_old_cache(state: &crate::YaffeState) {
    use crate::pooled_cache::PooledCacheIndex;
    let map = unsafe { FILE_ASSET_MAP.as_mut().unwrap() };

    let mut total_memory = 0;
    let mut last_used_index: Option<PooledCacheIndex> = None;
    let mut last_request = Instant::now();
    let indices = map.indexes().collect::<Vec<PooledCacheIndex>>();
    for index in indices {
        let slot = map.get_index_mut(index).unwrap();
        let mut slot = slot.borrow_mut();
        if slot.state.load(Ordering::Acquire) == ASSET_STATE_LOADED {
            total_memory += slot.data_length;

            //Find oldest asset
            if slot.last_request < last_request {
                last_request = slot.last_request;
                last_used_index = Some(index);
            } else if slot.last_request.elapsed().as_secs() > 60 {
                //If it hasnt been requested in a minute, remove it regardless
                slot.data = AssetData::None;
                slot.state.store(ASSET_STATE_UNLOADED, Ordering::Release);
            }
        }
    }
    //Remove oldest asset if we are over our memory threshold
    //This will happen once per frame until we are under the threshold
    if total_memory > 1024 * 1024 * state.settings.get_i32(crate::settings::SettingNames::AssetCacheSizeMb) as usize {
       if let Some(index) = last_used_index {
            let slot = map.get_index_mut(index).unwrap();
            let mut slot = slot.borrow_mut();
            slot.data = AssetData::None;
            slot.state.store(ASSET_STATE_UNLOADED, Ordering::Release);
       }
    }
}
