use speedy2d::font::*;
use std::sync::atomic::Ordering;
use std::time::Instant;
use crate::logger::PanicLogEntry;
use super::{AssetData, ASSET_STATE_LOADED, AssetTypes, get_asset_slot, AssetKey};

#[derive(PartialEq, Eq, Hash, Clone, Copy, Debug)]
pub enum Fonts {
    Regular,
}

pub fn request_font(font: Fonts) -> &'static Font {
    let slot = get_asset_slot(&AssetKey::Static(AssetTypes::Font(font)));

    assert!(slot.path.exists());
    assert_eq!(slot.state.load(Ordering::Acquire), ASSET_STATE_LOADED, "requested font, but font is not loaded");

    if let AssetData::Raw((data, _)) = &slot.data {
        let font = speedy2d::font::Font::new(data).log_and_panic();
        slot.data_length = data.len();
        slot.data = AssetData::Font(font);
    }

    if let AssetData::Font(font) = &slot.data {
        slot.last_request = Instant::now();
        return font;
    }
    panic!("Requested font on a non-font asset slot");
}
