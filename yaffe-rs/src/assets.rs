use std::sync::atomic::{AtomicU8, Ordering};
use std::rc::Rc;
use std::cell::RefCell;
use crate::{RawDataPointer, PhysicalSize, PhysicalRect};
use crate::logger::{PanicLogEntry, info, warn};
use speedy2d::{Graphics2D, font::*, image::*, color::Color};
use std::time::Instant;
use crate::pooled_cache::PooledCache;
use std::assert_matches::assert_matches;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Images {
    Background,
    Placeholder,
    PlaceholderBanner,
    Error,
    Question,
    ArrowUp,
    ArrowDown,
    ButtonA,
    ButtonB,
    ButtonX,
    ButtonY,
    App,
    Emulator,
    Recent,
    Speaker,
    Settings,
    ErsbEveryone,
    ErsbEveryone10,
    ErsbTeen,
    ErsbMature,
    ErsbAdultOnly
}

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Fonts {
    Regular,
}

const ASSET_STATE_UNLOADED: u8 = 0;
const ASSET_STATE_PENDING: u8 = 1;
const ASSET_STATE_LOADED: u8 = 2;

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum AssetTypes {
    Image(Images),
    Font(Fonts),
}

#[derive(Debug, Clone)]
pub enum AssetPathType {
    File(String),
    Url(String),
}

pub enum AssetData {
    Image(YaffeTexture),
    Font(Font),
}

pub struct AssetSlot {
    state: AtomicU8,
    path: AssetPathType,
    data: Vec<u8>,
    dimensions: (u32, u32),
    image: Option<AssetData>,
    last_request: Instant,
}
impl AssetSlot {
    pub fn new(path: AssetPathType) -> AssetSlot {
        AssetSlot {
            state: AtomicU8::new(ASSET_STATE_UNLOADED),
            path,
            data: Vec::with_capacity(0),
            dimensions: (0, 0),
            image: None,
            last_request: Instant::now(),
        }
    }

    pub fn preloaded(path: &str, image: YaffeTexture) -> AssetSlot {
        AssetSlot {
            state: AtomicU8::new(ASSET_STATE_LOADED),
            path: AssetPathType::File(String::from(path)),
            data: Vec::with_capacity(0),
            dimensions: (0, 0),
            image: Some(AssetData::Image(image)),
            last_request: Instant::now(),
        }
    }

    pub fn font(path: &str) -> AssetSlot {
        let data = std::fs::read(path).log_and_panic();
        let font = speedy2d::font::Font::new(&data).log_and_panic();

        AssetSlot {
            state: AtomicU8::new(ASSET_STATE_LOADED),
            path: AssetPathType::File(String::from(path)),
            data: Vec::with_capacity(0),
            dimensions: (0, 0),
            image: Some(AssetData::Font(font)),
            last_request: Instant::now(),
        }
    }
}
pub struct YaffeTexture {
    image: Rc<ImageHandle>,
    bounds: Option<PhysicalRect>,
}
impl YaffeTexture {
    pub fn render(&self, graphics: &mut crate::Graphics, rect: crate::Rect) {
        let rect = rect.to_physical(graphics.scale_factor);
        if let Some(b) = &self.bounds {
            graphics.graphics.draw_rectangle_image_subset_tinted(rect, Color::WHITE, b.clone(), &self.image);
        } else {
            graphics.graphics.draw_rectangle_image(rect, &self.image);
        }
    }

    pub fn render_tinted(&self, graphics: &mut crate::Graphics, color: Color, rect: crate::Rect) {
        let rect = rect.to_physical(graphics.scale_factor);
        graphics.graphics.draw_rectangle_image_tinted(rect, color, &self.image);
    }

    pub fn size(&self) -> PhysicalSize {
        let size = self.image.size();
        PhysicalSize::new(size.x as f32, size.y as f32)
    }
}

//Stores static assets (something from AssetTypes)
static mut STATIC_ASSET_MAP: Option<PooledCache<32, AssetTypes, AssetSlot>> = None;

//Stores dyanmic assets (something loaded from a path)
static mut FILE_ASSET_MAP: Option<PooledCache<32, String, RefCell<AssetSlot>>> = None;

pub fn initialize_asset_cache() {
    let mut map = PooledCache::new();
    map.insert(AssetTypes::Image(Images::PlaceholderBanner), AssetSlot::new(AssetPathType::File(String::from(r"./Assets/banner.png"))));
    map.insert(AssetTypes::Image(Images::Background), AssetSlot::new(AssetPathType::File(String::from(r"./Assets/background.jpg"))));

    map.insert(AssetTypes::Font(Fonts::Regular), AssetSlot::font("./Assets/Roboto-Regular.ttf"));
    
    unsafe { STATIC_ASSET_MAP = Some(map); }

    unsafe { FILE_ASSET_MAP = Some(PooledCache::new()); }
}

pub fn preload_assets(graphics: &mut Graphics2D) {
    let map = unsafe { STATIC_ASSET_MAP.as_mut().unwrap() };
    if let None = map.get_mut(&AssetTypes::Image(Images::Error)) {
        let data = graphics.create_image_from_file_path(None, ImageSmoothingMode::Linear,"./Assets/packed.png").log_and_panic();
        let image = Rc::new(data);

        for tex in read_texture_atlas("./Assets/atlas.tex") {
            let image_type = match tex.0.as_str() {
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
            };

            let texture = YaffeTexture { image: image.clone(), bounds: Some(tex.1) };
            map.insert(AssetTypes::Image(image_type), AssetSlot::preloaded("./Assets/packed.png", texture));
        }
    }

    if let None = map.get_mut(&AssetTypes::Image(Images::Placeholder)) {
        preload_image(graphics, "./Assets/placeholder.jpg", Images::Placeholder, map);
    }

    fn preload_image(graphics: &mut Graphics2D, path: &'static str, image_name: Images, map: &mut  PooledCache<32, AssetTypes, AssetSlot>) {
        let data = graphics.create_image_from_file_path(None, ImageSmoothingMode::Linear, path).log_and_panic();
        let image = Rc::new(data);
        let texture = YaffeTexture { image: image.clone(), bounds: None };
        map.insert(AssetTypes::Image(image_name), AssetSlot::preloaded(path, texture));
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

pub fn request_asset_image<'a>(graphics: &mut crate::Graphics, slot: &'a mut AssetSlot) -> Option<&'a YaffeTexture> {
    if slot.state.load(Ordering::Acquire) == ASSET_STATE_UNLOADED && asset_path_is_valid(&slot.path) {
        if let Ok(ASSET_STATE_UNLOADED) = slot.state.compare_exchange(ASSET_STATE_UNLOADED, ASSET_STATE_PENDING, Ordering::Acquire, Ordering::Relaxed) {

            if let Some(queue) = &graphics.queue {
                let lock = queue.lock().log_and_panic();
                let mut queue = lock.borrow_mut();
                queue.send(crate::JobType::LoadImage((slot.path.clone(), RawDataPointer::new(slot))));
            }
            return None;
        }
    }

    if let None = slot.image {
        if slot.state.load(Ordering::Acquire) == ASSET_STATE_LOADED {
            let image = graphics.graphics.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::Linear, slot.dimensions, &slot.data).log_and_panic();
            slot.image = Some(AssetData::Image(YaffeTexture { image: Rc::new(image), bounds: None }));
            slot.data = Vec::with_capacity(0);
        }
    }

    if let Some(AssetData::Image(image)) = slot.image.as_ref() {
        slot.last_request = Instant::now();
        Some(image)
    } else {
        None
    }
}

pub fn request_image<'a>(piet: &mut crate::Graphics, image: Images) -> Option<&'a YaffeTexture> {
    let slot = get_slot_mut(AssetTypes::Image(image));

    request_asset_image(piet, slot)
}

pub fn request_font(font: Fonts) -> &'static Font {
    let slot = get_slot_mut(AssetTypes::Font(font));

    assert_matches!(&slot.path, AssetPathType::File(path) if std::path::Path::new(&path).exists());
    assert_eq!(slot.state.load(Ordering::Acquire), ASSET_STATE_LOADED, "requested preloaded image, but image is not loaded");

    if let None = slot.image {
        let font = speedy2d::font::Font::new(&slot.data).log_and_panic();
        slot.image = Some(AssetData::Font(font));
    }

    if let Some(AssetData::Font(font)) = slot.image.as_ref() {
        slot.last_request = Instant::now();
        return font;
    }
    panic!("Requested font on a non-font asset slot");
}

pub fn load_image_async(path: AssetPathType, slot: RawDataPointer) {
    info!("Loading image asynchronously {:?}", path);

    let data = match &path {
        AssetPathType::File(path) => std::fs::read(&path).log_and_panic(),
        AssetPathType::Url(path) =>  {
            let image = reqwest::blocking::get(path).unwrap().bytes().log_and_panic();
            image.to_vec()
        },
    };

    let mut reader = image::io::Reader::new(std::io::Cursor::new(data.clone()));
    reader = reader.with_guessed_format().log_and_panic();

    match reader.decode() {
        Ok(image) => {
            let buffer = image.into_rgba8();
            let asset_slot = slot.get_inner::<AssetSlot>();
            asset_slot.dimensions = buffer.dimensions();
            asset_slot.data = buffer.into_vec();
            asset_slot.state.swap(ASSET_STATE_LOADED, Ordering::AcqRel);
        },
        Err(e) => warn!("Error loading {:?}: {:?}", path, e),
    }
}

pub fn get_asset_path(platform: &str, name: &str) -> (std::path::PathBuf, std::path::PathBuf) {
    use std::path::Path;

    let platform = Path::new("./Assets").join(crate::platform_layer::sanitize_file(platform));
    let name = Path::new(&platform).join(crate::platform_layer::sanitize_file(name));
    if !platform.exists() { std::fs::create_dir(platform).unwrap(); }

    let banner = Path::new(&name).join("banner.jpg");
    let boxart = Path::new(&name).join("boxart.jpg");
    if !name.exists() { std::fs::create_dir(name).log_and_panic(); }

    (boxart.to_owned(), banner.to_owned())
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
    map.insert(key.clone(), RefCell::new(AssetSlot::new(file.clone())));
    map.get_mut(key).unwrap()
}

pub fn clear_old_cache(state: &crate::YaffeState) {
    use crate::pooled_cache::PooledCacheIndex;
    let map = unsafe { FILE_ASSET_MAP.as_mut().unwrap() };

    let mut to_remove = vec!();
    let mut total_memory = 0;
    let mut last_used_index: Option<PooledCacheIndex> = None;
    let mut last_request = Instant::now();
    let indices = map.indexes().collect::<Vec<PooledCacheIndex>>();
    for index in indices {
        let slot = map.get_index_mut(index).unwrap();
        let slot = slot.borrow();
        if slot.state.load(Ordering::Acquire) == ASSET_STATE_LOADED {
            total_memory += slot.data.len();

            //Find oldest asset
            if slot.last_request < last_request {
                last_request = slot.last_request;
                last_used_index = Some(index);
            } else if slot.last_request.elapsed().as_secs() > 60 {
                //If it hasnt been requested in a minute, remove it regardless
                to_remove.push(index);
            }
        }
    }
    //Remove oldest asset if we are over our memory threshold
    //This will happen once per frame until we are under the threshold
    if total_memory > 1024 * 1024 * state.settings.get_i32(crate::settings::SettingNames::AssetCacheSizeMb) as usize &&
        last_used_index.is_some() {
        to_remove.push(last_used_index.unwrap());
    }

    for i in to_remove {
        map.remove_at(i);
    }
}

fn read_texture_atlas(path: &str) -> Vec<(String, PhysicalRect)> {
    use std::convert::TryInto;

    macro_rules! read_type {
        ($ty:ty, $file:expr, $index:expr) => {{
                let size = std::mem::size_of::<$ty>();
                let value = <$ty>::from_le_bytes($file[$index..($index + size)].try_into().unwrap());
                $index += size;
                value
            }};
    }

    let file = std::fs::read(path).log_and_panic();
    let mut index = 0;
    let total_width = read_type!(i32, file, index) as f32;
    let total_height = read_type!(i32, file, index) as f32;
    let count = read_type!(i32, file, index);

    let mut result = vec!();
    for _ in 0..count {
        let mut name = String::from("");
        loop {
            let c = read_type!(u8, file, index);
            if c == 0 || index >= file.len() { break; }

            name.push(c as char);
        }

        let image_width = read_type!(i32, file, index);
        let image_height = read_type!(i32, file, index);
        let x = read_type!(i32, file, index);
        let y = read_type!(i32, file, index);

        let width = (x + image_width) as f32 / total_width;
        let height = (y + image_height) as f32 / total_height;
        let x = x as f32 / total_width;
        let y = y as f32 / total_height;
        result.push((name, PhysicalRect::from_tuples((x, y), (width, height))));
    }

    result
}