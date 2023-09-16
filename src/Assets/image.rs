use speedy2d::{image::*, color::Color, Graphics2D};
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::time::Instant;
use crate::{PhysicalSize, PhysicalRect};
use crate::logger::{PanicLogEntry, warn, info};
use crate::pooled_cache::PooledCache;
use super::{AssetData, ASSET_STATE_LOADED, AssetKey, AssetSlot};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Images {
    Background,
    Placeholder,
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

pub struct YaffeTexture {
    image: Rc<ImageHandle>,
    bounds: Option<PhysicalRect>,
}
impl YaffeTexture {
    pub fn new(image: Rc<ImageHandle>, bounds: Option<PhysicalRect>) -> YaffeTexture {
        YaffeTexture { image, bounds }
    }

    pub fn render(&self, graphics: &mut crate::Graphics, rect: crate::Rect) {
        let rect = rect.to_physical(graphics.scale_factor);
        if let Some(b) = &self.bounds {
            graphics.draw_rectangle_image_subset_tinted(rect, Color::WHITE, b.clone(), &self.image);
        } else {
            graphics.draw_rectangle_image(rect, &self.image);
        }
    }

    pub fn render_tinted(&self, graphics: &mut crate::Graphics, color: Color, rect: crate::Rect) {
        let rect = rect.to_physical(graphics.scale_factor);
        graphics.draw_rectangle_image_tinted(rect, color, &self.image);
    }

    pub fn size(&self) -> PhysicalSize {
        let size = self.image.size();
        PhysicalSize::new(size.x as f32, size.y as f32)
    }
}

pub fn request_asset_image<'a>(graphics: &mut crate::Graphics, key: &AssetKey) -> Option<&'a YaffeTexture> {
    let q = graphics.queue.clone();
    let queue = q.as_ref();
    let lock = queue.lock().log_and_panic();
    let mut queue = lock.borrow_mut();

    if let Some(slot) = super::ensure_asset_loaded(&mut queue, key) {
        if slot.state.load(Ordering::Acquire) == ASSET_STATE_LOADED {
            if let AssetData::Raw((data, dimensions)) = &slot.data {
                let image = graphics.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::Linear, *dimensions, data).log_and_panic();
                slot.data = AssetData::Image(YaffeTexture { image: Rc::new(image), bounds: None });
            }
        }
    
        return if let AssetData::Image(image) = &slot.data {
            slot.last_request = Instant::now();
            Some(image)
        } else {
            None
        };
    }
    None
}

pub fn request_image<'a>(graphics: &mut crate::Graphics, image: Images) -> Option<&'a YaffeTexture> {
    request_asset_image(graphics, &AssetKey::image(image))
}

pub fn load_image_async(key: &AssetKey, path: std::path::PathBuf) -> Option<(Vec<u8>, (u32, u32))> {
    info!("Loading image asynchronously {:?}", path);

    let data = match &key {
        AssetKey::File(_) | AssetKey::Static(_) => std::fs::read(path).log_and_panic(),
        AssetKey::Url(_) =>  {
            let image = reqwest::blocking::get(path.to_str().unwrap()).unwrap().bytes().log_and_panic();
            image.to_vec()
        },
    };

    let mut reader = image::io::Reader::new(std::io::Cursor::new(data));
    reader = reader.with_guessed_format().log_and_panic();

    match reader.decode() {
        Ok(image) => {
            let buffer = image.into_rgba8();
            let dimensions = buffer.dimensions();
            let data = buffer.into_vec();
            return Some((data, dimensions))
        },
        Err(e) => warn!("Error loading {:?}: {:?}", key, e),
    }
    None
}

pub fn preload_image(graphics: &mut Graphics2D, path: &'static str, image_name: Images, map: &mut  PooledCache<32, AssetKey, AssetSlot>) {
    let data = graphics.create_image_from_file_path(None, ImageSmoothingMode::Linear, path).log_and_panic();
    let image = Rc::new(data);
    let texture = YaffeTexture::new(image, None);
    map.insert(AssetKey::image(image_name), AssetSlot::preloaded(path, texture));
}
