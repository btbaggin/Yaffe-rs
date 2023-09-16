use std::convert::TryInto;
use std::rc::Rc;
use speedy2d::image::ImageHandle;
use crate::pooled_cache::PooledCache;
use crate::PhysicalRect;
use crate::logger::PanicLogEntry;
use super::{AssetSlot, YaffeTexture, AssetKey, Images};

macro_rules! read_type {
    ($ty:ty, $file:expr, $index:expr) => {{
            let size = std::mem::size_of::<$ty>();
            let value = <$ty>::from_le_bytes($file[$index..($index + size)].try_into().unwrap());
            $index += size;
            value
        }};
}

pub fn load_texture_atlas<F>(map: &mut PooledCache<32, AssetKey, AssetSlot>, image: Rc<ImageHandle>, path: &str, image_path: &str, image_map: F)
    where F: Fn(&str) -> Images {

    let file = std::fs::read(path).log_and_panic();
    let mut index = 0;
    let total_width = read_type!(i32, file, index) as f32;
    let total_height = read_type!(i32, file, index) as f32;
    let count = read_type!(i32, file, index);

    for _ in 0..count {
        let mut name = String::from("");
        loop {
            let c = read_type!(u8, file, index);
            if c == 0 || index >= file.len() { break; }

            name.push(c as char);
        }

        let image_width = read_type!(i32, file, index) as f32;
        let image_height = read_type!(i32, file, index) as f32;
        let x = read_type!(i32, file, index) as f32;
        let y = read_type!(i32, file, index) as f32;

        let width = (x + image_width) / total_width;
        let height = (y + image_height) / total_height;
        let x = x / total_width;
        let y = y / total_height;
        let bounds = Some(PhysicalRect::from_tuples((x, y), (width, height)));

        let image_type = image_map(name.as_str());
        let texture = YaffeTexture::new(image.clone(), bounds);
        map.insert(AssetKey::image(image_type), AssetSlot::preloaded(image_path, texture));
    }
}
