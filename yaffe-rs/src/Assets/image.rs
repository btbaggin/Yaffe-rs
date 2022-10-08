use speedy2d::{image::*, color::Color, Graphics2D};
use std::rc::Rc;
use std::sync::atomic::Ordering;
use std::time::Instant;
use crate::{PhysicalSize, PhysicalRect};
use crate::RawDataPointer;
use crate::logger::{PanicLogEntry, warn, info};
use crate::pooled_cache::PooledCache;
use super::{AssetData, ASSET_STATE_LOADED, ASSET_STATE_PENDING, ASSET_STATE_UNLOADED, AssetTypes, get_slot_mut, asset_path_is_valid, AssetPathType, AssetSlot};

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

pub struct YaffeTexture {
    pub(super) image: Rc<ImageHandle>,
    pub(super) bounds: Option<PhysicalRect>,
}
impl YaffeTexture {
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

pub fn request_asset_image<'a>(graphics: &mut crate::Graphics, slot: &'a mut AssetSlot) -> Option<&'a YaffeTexture> {
    if slot.state.load(Ordering::Acquire) == ASSET_STATE_UNLOADED && asset_path_is_valid(&slot.path) {
        if let Ok(ASSET_STATE_UNLOADED) = slot.state.compare_exchange(ASSET_STATE_UNLOADED, ASSET_STATE_PENDING, Ordering::Acquire, Ordering::Relaxed) {

            if let Some(queue) = &graphics.queue {
                let lock = queue.lock().log_and_panic();
                let mut queue = lock.borrow_mut();
                queue.send(crate::JobType::LoadImage((slot.path.clone(), RawDataPointer::new(slot)))).unwrap();
            }
            return None;
        }
    }

    if slot.state.load(Ordering::Acquire) == ASSET_STATE_LOADED {
        if let AssetData::Raw(data) = &slot.data {
            let image = graphics.create_image_from_raw_pixels(ImageDataType::RGBA, ImageSmoothingMode::Linear, slot.dimensions, &data).log_and_panic();
            slot.data = AssetData::Image(YaffeTexture { image: Rc::new(image), bounds: None });
        }
    }

    if let AssetData::Image(image) = &slot.data {
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
            asset_slot.data = AssetData::Raw(buffer.into_vec());
            asset_slot.state.swap(ASSET_STATE_LOADED, Ordering::AcqRel);
        },
        Err(e) => warn!("Error loading {:?}: {:?}", path, e),
    }
}

pub fn preload_image(graphics: &mut Graphics2D, path: &'static str, image_name: Images, map: &mut  PooledCache<32, AssetTypes, AssetSlot>) {
    let data = graphics.create_image_from_file_path(None, ImageSmoothingMode::Linear, path).log_and_panic();
    let image = Rc::new(data);
    let texture = YaffeTexture { image: image.clone(), bounds: None };
    map.insert(AssetTypes::Image(image_name), AssetSlot::preloaded(path, texture));
}
