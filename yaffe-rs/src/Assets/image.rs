use super::{AssetData, AssetKey, AssetSlot, ASSET_STATE_LOADED};
use crate::graphics::Graphics;
use crate::logger::{info, warn, PanicLogEntry};
use crate::pooled_cache::PooledCache;
use crate::{PhysicalRect, PhysicalSize};
use speedy2d::{image::*, Graphics2D};
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::time::Instant;

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
    ErsbAdultOnly,
}

#[derive(Clone)]
pub struct YaffeTexture {
    pub image: Rc<ImageHandle>,
    pub bounds: Option<PhysicalRect>,
}
impl YaffeTexture {
    pub fn new(image: Rc<ImageHandle>, bounds: Option<PhysicalRect>) -> YaffeTexture { YaffeTexture { image, bounds } }

    pub fn size(&self) -> PhysicalSize {
        let size = self.image.size();
        PhysicalSize::new(size.x as f32, size.y as f32)
    }
}
impl Graphics {
    pub fn request_asset_image(&mut self, key: &AssetKey) -> Option<YaffeTexture> {
        let queue = self.queue.clone();
        let mut map = self.asset_cache.borrow_mut();
        if let Some(slot) = super::ensure_asset_loaded(queue, &mut map, key) {
            if slot.state.load(Ordering::Acquire) == ASSET_STATE_LOADED {
                if let AssetData::Raw((data, dimensions)) = &slot.data {
                    let graphics = unsafe { &mut *self.graphics_ptr };
                    let image = graphics
                        .create_image_from_raw_pixels(
                            ImageDataType::RGBA,
                            ImageSmoothingMode::Linear,
                            *dimensions,
                            data,
                        )
                        .log_and_panic();
                    slot.data = AssetData::Image(YaffeTexture { image: Rc::new(image), bounds: None });
                }
            }

            return if let AssetData::Image(image) = &slot.data {
                slot.last_request = Instant::now();
                Some(image.clone())
            } else {
                None
            };
        }
        None
    }

    pub fn request_image(&mut self, image: Images) -> Option<YaffeTexture> {
        self.request_asset_image(&AssetKey::image(image))
    }
}

pub fn load_image_async(key: &AssetKey, path: std::path::PathBuf) -> Option<(Vec<u8>, (u32, u32))> {
    info!("Loading image asynchronously {path:?}");

    let data = match &key {
        AssetKey::File(_) | AssetKey::Static(_) => std::fs::read(path).log_and_panic(),
        AssetKey::Url(_) => {
            let image = reqwest::blocking::get(path.to_str().unwrap()).unwrap().bytes().log_and_panic();
            image.to_vec()
        }
    };

    let mut reader = image::io::Reader::new(std::io::Cursor::new(data));
    reader = reader.with_guessed_format().log_and_panic();

    match reader.decode() {
        Ok(image) => {
            let buffer = image.into_rgba8();
            let dimensions = buffer.dimensions();
            let data = buffer.into_vec();
            return Some((data, dimensions));
        }
        Err(e) => warn!("Error loading {key:?}: {e:?}"),
    }
    None
}

pub fn preload_image(
    graphics: &mut Graphics2D,
    path: &'static str,
    image_name: Images,
    map: &mut PooledCache<32, AssetKey, AssetSlot>,
) {
    let data = graphics.create_image_from_file_path(None, ImageSmoothingMode::Linear, path).log_and_panic();
    let image = Rc::new(data);
    let texture = YaffeTexture::new(image, None);
    map.insert(AssetKey::image(image_name), AssetSlot::preloaded(path, texture));
}
