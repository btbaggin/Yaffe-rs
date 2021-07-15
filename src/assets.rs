use std::collections::HashMap;
use std::sync::atomic::{AtomicU8, Ordering};
use std::rc::Rc;
use std::cell::RefCell;
use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::V2;
use crate::job_system::JobQueue;
use crate::logger::LogEntry;
use speedy2d::font::*;
use speedy2d::image::*;

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

const ASSET_STATE_UNLOADED: u8 = 0;
const ASSET_STATE_PENDING: u8 = 1;
const ASSET_STATE_LOADED: u8 = 2;

pub struct AssetSlot {
    state: AtomicU8,
    path: String,
    data: Option<((u32, u32), Vec<u8>)>,
    image: Option<YaffeTexture>,
}
impl AssetSlot {
    pub fn new(path: &str) -> AssetSlot {
        AssetSlot {
            state: AtomicU8::new(ASSET_STATE_UNLOADED),
            path: String::from(path),
            data: None,
            image: None,
        }
    }

    pub fn packed_texture(path: &str, image: YaffeTexture) -> AssetSlot {
        AssetSlot {
            state: AtomicU8::new(ASSET_STATE_LOADED),
            path: String::from(path),
            data: None,
            image: Some(image),
        }
    }

    pub fn get_image_size(&self) -> Option<V2> {
        if let Some(i) = &self.image {
            return Some(i.size());
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

static mut ASSET_MAP: Option<HashMap<Images, AssetSlot>> = None;
static mut FILE_ASSET_MAP: Option<HashMap<String, Rc<RefCell<AssetSlot>>>> = None;

pub fn initialize_asset_cache() {
    let mut map = HashMap::new();
    map.insert(Images::Placeholder, AssetSlot::new(r"./Assets/placeholder.jpg"));
    map.insert(Images::PlaceholderBanner, AssetSlot::new(r"./Assets/banner.png"));
    map.insert(Images::Background, AssetSlot::new(r"./Assets/background.jpg"));
    
    unsafe { ASSET_MAP = Some(map); }

    unsafe { FILE_ASSET_MAP = Some(HashMap::new()); }
}

pub fn load_texture_atlas(piet: &mut Graphics2D) {
    //TODO put behind a dynamic load?
    let map = unsafe { ASSET_MAP.as_mut().unwrap() };
    if let None = map.get(&Images::Error) {
        let data = piet.create_image_from_file_path(None, ImageSmoothingMode::Linear,"./Assets/packed.png").log_if_fail();
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
            map.insert(image_type, AssetSlot::packed_texture("./Assets/packed.png", texture));
        }
    }
}

pub fn request_asset_image<'a>(piet: &mut Graphics2D, queue: &mut JobQueue, slot: &'a mut AssetSlot) -> Option<&'a YaffeTexture> {
    if slot.state.load(Ordering::Relaxed) == ASSET_STATE_UNLOADED && 
       std::path::Path::new(&slot.path).exists() {
        if let Ok(ASSET_STATE_UNLOADED) = slot.state.compare_exchange(ASSET_STATE_UNLOADED, ASSET_STATE_PENDING, Ordering::Acquire, Ordering::Relaxed) {

            queue.send(crate::JobType::LoadImage(crate::RawDataPointer::new(slot)));
            return None;
        }
    }

    if let None = slot.image {
        if let Some((dimension, data)) = &slot.data {
            let image = piet.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::Linear, *dimension, data).log_if_fail();
            slot.image = Some(YaffeTexture { image: Rc::new(image), bounds: None });
        }
    }

    slot.image.as_ref()  
}

pub fn request_image<'a>(piet: &mut Graphics2D, queue: &mut JobQueue, image: Images) -> Option<&'a YaffeTexture> {
    let slot = unsafe { ASSET_MAP.as_mut().unwrap().get_mut(&image).log_message_if_fail("Requesting image that was never added to asset map") };

    request_asset_image(piet, queue, slot)
}

pub fn request_preloaded_image<'a>(piet: &mut Graphics2D, image: Images) -> &'a YaffeTexture {
    let slot = unsafe { ASSET_MAP.as_mut().unwrap().get_mut(&image).log_message_if_fail("Requesting image that was never added to asset map") };

    assert_eq!(std::path::Path::new(&slot.path).exists(), true);
    assert_eq!(slot.state.load(Ordering::Relaxed), ASSET_STATE_LOADED, "requested preloaded image, but image is not loaded");

    if let None = slot.image {
        if let Some((dimension, data)) = &slot.data {
            let image = piet.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::Linear, *dimension, data).log_if_fail();
            slot.image = Some(YaffeTexture { image: Rc::new(image), bounds: None });
        }
    }

    slot.image.as_ref().unwrap()  
}

//TODO font families?
static mut FONT: Option<Font> = None;
pub fn request_font() -> Font {
    //TODO load?
    if let None = unsafe { &FONT } {
        let data = std::fs::read("./Assets/Roboto-Regular.ttf").log_if_fail();
        let font = speedy2d::font::Font::new(&data).log_if_fail();
        unsafe { FONT = Some(font); }
    }

    unsafe { FONT.as_ref().unwrap().clone() }
}

pub fn load_image_async(slot: crate::RawDataPointer) {
    use image::GenericImageView;
    let asset_slot = slot.get_inner::<AssetSlot>();
    let data = std::fs::read(&asset_slot.path).log_if_fail();

    let mut reader = image::io::Reader::new(std::io::Cursor::new(data));
    reader = reader.with_guessed_format().log_if_fail();

    let image = reader.decode().log_if_fail();
    let dimensions = image.dimensions();
    let bytes_rgba8 = image.into_rgba8().into_raw();

    asset_slot.data = Some((dimensions, bytes_rgba8));
    asset_slot.state.swap(ASSET_STATE_LOADED, Ordering::Relaxed);
    //TODO i could write something to indicate we need to draw a new frame?
}

pub fn get_asset_path(platform: &str, name: &str) -> (String, String) {
    use std::path::Path;

    let platform = Path::new("./Assets").join(platform);
    let name = Path::new(&platform).join(name);
    if !platform.exists() { std::fs::create_dir(platform).unwrap(); }

    let banner = Path::new(&name).join("banner.jpg");
    let boxart = Path::new(&name).join("boxart.jpg");
    if !name.exists() { std::fs::create_dir(name).log_if_fail(); }

    (boxart.to_string_lossy().to_string(), banner.to_string_lossy().to_string())
}

pub fn get_asset_slot(platform: &str, name: &str) -> (Rc<RefCell<AssetSlot>>, Rc<RefCell<AssetSlot>>) {
    let (boxart, banner) = get_asset_path(platform, name);

    //This acts as a cache of exe images
    //If our list ever reloads or we reqeust the same image (recent vs emulator)
    //We will grab the cached image so we dont need to reload the image data
    let map = unsafe { FILE_ASSET_MAP.as_mut().unwrap() };
    if let None = map.get(&boxart) { 
        map.insert(boxart.clone(), Rc::new(RefCell::new(AssetSlot::new(&boxart)))); 
    } 
    if let None = map.get(&banner) { 
        map.insert(banner.clone(), Rc::new(RefCell::new(AssetSlot::new(&banner)))); 
    } 

    let boxart = map.get(&boxart).unwrap();
    let banner = map.get(&banner).unwrap();

    (boxart.clone(), banner.clone())
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

    let file = std::fs::read(path).log_if_fail();
    let mut index = 0;
    let total_width = read_type!(i32, file, index);
    let total_height = read_type!(i32, file, index);
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

        let width = (x + image_width) as f32 / total_width as f32;
        let height = (y + image_height) as f32 / total_height as f32;
        let x = x as f32 / total_width as f32;
        let y = y as f32 / total_height as f32;
        result.push((name, Rectangle::from_tuples((x, y), (width, height))));
    }

    result
}