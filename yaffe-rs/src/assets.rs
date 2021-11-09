use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::rc::Rc;
use std::cell::RefCell;
use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::V2;
use crate::job_system::JobQueue;
use crate::logger::PanicLogEntry;
use speedy2d::font::*;
use speedy2d::image::*;
use std::time::Instant;
// use std::assert_matches::assert_matches;

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

    pub fn packed_texture(path: &str, image: YaffeTexture) -> AssetSlot {
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

    pub fn get_image_size(&self) -> Option<V2> {
        if let Some(texture) = &self.image {
            if let AssetData::Image(i) = texture {
                return Some(i.size());
            }
        }
        None
    }
}
pub struct YaffeTexture {
    image: Rc<ImageHandle>,
    bounds: Option<Rectangle>,
}
impl YaffeTexture {
    pub fn render(&self, piet: &mut Graphics2D, rect: Rectangle) {
        if let Some(b) = &self.bounds {
            piet.draw_rectangle_image_subset_tinted(rect, speedy2d::color::Color::WHITE, b.clone(), &self.image);
        } else {
            piet.draw_rectangle_image(rect, &self.image);
        }
    }

    pub fn size(&self) -> V2 {
        let size = self.image.size();
        V2::new(size.x as f32, size.y as f32)
    }

    pub fn get_handle(&self) -> &Rc<ImageHandle> { &self.image }
}

static mut STATIC_ASSET_MAP: Option<HashMap<AssetTypes, AssetSlot>> = None;
static mut FILE_ASSET_MAP: Option<HashMap<String, RefCell<AssetSlot>>> = None;

pub fn initialize_asset_cache() {
    let mut map = HashMap::new();
    map.insert(AssetTypes::Image(Images::Placeholder), AssetSlot::new(AssetPathType::File(String::from(r"./Assets/placeholder.jpg"))));
    map.insert(AssetTypes::Image(Images::PlaceholderBanner), AssetSlot::new(AssetPathType::File(String::from(r"./Assets/banner.png"))));
    map.insert(AssetTypes::Image(Images::Background), AssetSlot::new(AssetPathType::File(String::from(r"./Assets/background.jpg"))));

    map.insert(AssetTypes::Font(Fonts::Regular), AssetSlot::font("./Assets/Roboto-Regular.ttf"));
    
    unsafe { STATIC_ASSET_MAP = Some(map); }

    unsafe { FILE_ASSET_MAP = Some(HashMap::with_capacity(64)); }
}

pub fn load_texture_atlas(graphics: &mut Graphics2D) {
    let map = unsafe { STATIC_ASSET_MAP.as_mut().unwrap() };
    if let None = map.get(&AssetTypes::Image(Images::Error)) {
        let data = graphics.create_image_from_file_path(None, ImageSmoothingMode::Linear,"./Assets/packed.png").log_and_panic();
        let image = Rc::new(data);

        for tex in read_texture_atlas(r"./Assets/atlas.tex") {
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
            map.insert(AssetTypes::Image(image_type), AssetSlot::packed_texture("./Assets/packed.png", texture));
        }
    }
}

fn get_slot_mut(t: AssetTypes) -> &'static mut AssetSlot {
    unsafe { STATIC_ASSET_MAP.as_mut().unwrap().get_mut(&t).log_message_and_panic("Invalid asset slot reqeust") }
}

fn asset_path_is_valid(path: &AssetPathType) -> bool {
    match path {
        AssetPathType::File(p) => std::path::Path::new(&p).exists(),
        AssetPathType::Url(_) => true,
    }
}

pub fn request_asset_image<'a>(piet: &mut Graphics2D, queue: &mut JobQueue, slot: &'a mut AssetSlot) -> Option<&'a YaffeTexture> {
    if slot.state.load(Ordering::Acquire) == ASSET_STATE_UNLOADED && 
       asset_path_is_valid(&slot.path) {
        if let Ok(ASSET_STATE_UNLOADED) = slot.state.compare_exchange(ASSET_STATE_UNLOADED, ASSET_STATE_PENDING, Ordering::Acquire, Ordering::Relaxed) {

            queue.send(crate::JobType::LoadImage(crate::RawDataPointer::new(slot)));
            return None;
        }
    }

    if let None = slot.image {
        if slot.state.load(Ordering::Acquire) == ASSET_STATE_LOADED {
            let image = piet.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::Linear, slot.dimensions, &slot.data).log_and_panic();
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

pub fn request_image<'a>(piet: &mut Graphics2D, queue: &mut JobQueue, image: Images) -> Option<&'a YaffeTexture> {
    let slot = get_slot_mut(AssetTypes::Image(image));

    request_asset_image(piet, queue, slot)
}

pub fn request_preloaded_image<'a>(piet: &mut Graphics2D, image: Images) -> &'a YaffeTexture {
    let slot = get_slot_mut(AssetTypes::Image(image));

    //TODO 
    //assert_matches!(slot.path, AssetPathType::File(path) if std::path::Path::new(&path).exists());
    assert_eq!(slot.state.load(Ordering::Relaxed), ASSET_STATE_LOADED, "requested preloaded image, but image is not loaded");

    if let None = slot.image {
        let image = piet.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::Linear, slot.dimensions, &slot.data).log_and_panic();
        slot.image = Some(AssetData::Image(YaffeTexture { image: Rc::new(image), bounds: None }));
    }

    if let Some(AssetData::Image(image)) = slot.image.as_ref() {
        slot.last_request = Instant::now();
        return image;
    }
    panic!("Requested image on a non-image asset slot");
}

pub fn request_font(font: Fonts) -> &'static Font {
    let slot = get_slot_mut(AssetTypes::Font(font));

    //TODO
    //assert_matches!(slot.path, AssetPathType::File(path) if std::path::Path::new(&slot.path).exists())
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

pub fn load_image_async(slot: crate::RawDataPointer) {
    let asset_slot = slot.get_inner::<AssetSlot>();
    let data = match &asset_slot.path {
        AssetPathType::File(path) => std::fs::read(&path).log_and_panic(),
        AssetPathType::Url(path) =>  {
            let image = reqwest::blocking::get(path).unwrap().bytes().log_and_panic();
            image.to_vec()
        },
    };

    let mut reader = image::io::Reader::new(std::io::Cursor::new(data.clone()));
    reader = reader.with_guessed_format().log_and_panic();

    let image = reader.decode().log_and_panic();
    let buffer = image.into_rgba8();

    asset_slot.dimensions = buffer.dimensions();
    asset_slot.data = buffer.into_vec();
    asset_slot.state.swap(ASSET_STATE_LOADED, Ordering::AcqRel);
}

pub fn get_asset_path(platform: &str, name: &str) -> (String, String) {
    use std::path::Path;

    let platform = Path::new("./Assets").join(platform);
    let name = Path::new(&platform).join(name);
    if !platform.exists() { std::fs::create_dir(platform).unwrap(); }

    let banner = Path::new(&name).join("banner.jpg");
    let boxart = Path::new(&name).join("boxart.jpg");
    if !name.exists() { std::fs::create_dir(name).log_and_panic(); }

    (boxart.to_string_lossy().to_string(), banner.to_string_lossy().to_string())
}

pub fn get_cached_file<'a>(file: &AssetPathType) -> &'a RefCell<AssetSlot> {

    //This acts as a cache of exe images
    //If our list ever reloads or we reqeust the same image (recent vs emulator)
    //We will grab the cached image so we dont need to reload the image data

    //TODO safety: since we pass was pointers to AssetSlots to background threads
    //if this Hashmap resizes and moves things around while an image is loading
    //that image can be loaded into random memory
    let map = unsafe { FILE_ASSET_MAP.as_mut().unwrap() };
    let key = match file {
        AssetPathType::File(path) => path,
        AssetPathType::Url(url) => url,
    };
    if !map.contains_key(key) {
        map.insert(key.clone(), RefCell::new(AssetSlot::new(file.clone())));
    }
    map.get(key).unwrap()
}

pub fn clear_old_cache(state: &crate::YaffeState) {
    let map = unsafe { FILE_ASSET_MAP.as_mut().unwrap() };

    let mut to_remove = vec!();
    let mut total_memory = 0;
    let mut last_used = ("", Instant::now());
    for (key, value) in map.iter() {
        let slot = value.borrow();
        if slot.state.load(Ordering::Acquire) == ASSET_STATE_LOADED {
            total_memory += slot.data.len();

            //Find oldest asset
            if slot.last_request < last_used.1 {
                last_used = (&key, slot.last_request);
            } else if slot.last_request.elapsed().as_secs() > 60 {
                //If it hasnt been requested in a minute, remove it regardless
                to_remove.push(key.clone());
            }
        }
    }

    //Remove oldest asset if we are over our memory threshold
    //This will happen once per frame until we are under the threshold
    if total_memory > 1024 * 1024 * state.settings.get_i32(crate::settings::SettingNames::AssetCacheSizeMb) as usize {
        to_remove.push(last_used.0.to_string());
    }

    for r in to_remove {
        map.remove(&r);
    }
}

fn read_texture_atlas(path: &str) -> Vec<(String, Rectangle)> {
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
        result.push((name, Rectangle::from_tuples((x, y), (width, height))));
    }

    result
}