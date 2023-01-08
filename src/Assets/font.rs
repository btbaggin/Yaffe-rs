use speedy2d::font::*;
use std::sync::atomic::Ordering;
use std::time::Instant;
use std::assert_matches::assert_matches;
use crate::logger::PanicLogEntry;
use super::{AssetData, ASSET_STATE_LOADED, AssetTypes, get_slot_mut, AssetPathType};

#[derive(PartialEq, Eq, Hash, Clone, Copy)]
pub enum Fonts {
    Regular,
}

pub fn request_font(font: Fonts) -> &'static Font {
    let slot = get_slot_mut(AssetTypes::Font(font));

    assert_matches!(&slot.path, AssetPathType::File(path) if std::path::Path::new(&path).exists());
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
