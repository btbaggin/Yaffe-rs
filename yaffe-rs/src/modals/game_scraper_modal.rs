
use super::{ModalResult, ModalContent, ListModal, default_modal_action, modal_width, ModalSize};
use crate::{YaffeState, Rect, LogicalSize, LogicalPosition};
use crate::net_api::GameScrapeResult;
use crate::settings::SettingsFile;
use crate::input::Actions;
use crate::assets::{AssetSlot, AssetPathType, request_asset_image};
use std::cell::RefCell;
use crate::ui_control::List;


pub struct GameScraperModal {
    list: List<GameScrapeResult>,
    slot: RefCell<AssetSlot>,
}
impl GameScraperModal {
    pub fn new(items: Vec<GameScrapeResult>) -> GameScraperModal {
        let path = items[0].boxart.clone();
        let path = std::path::Path::new("https://cdn.thegamesdb.net/images/medium/").join(path);
        GameScraperModal {
            list: List::new(items),
            slot: RefCell::new(AssetSlot::new(AssetPathType::Url(path.to_string_lossy().to_string())))
        }
    }
}

impl ModalContent for GameScraperModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, _: &SettingsFile, rect: Rect, _: &crate::Graphics) -> LogicalSize {
        LogicalSize::new(modal_width(rect, ModalSize::Full), rect.height())
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        if self.list.update(action) {
            let item = self.list.get_selected();
            let path = std::path::Path::new("https://cdn.thegamesdb.net/images/medium/").join(item.boxart.clone());
            self.slot = RefCell::new(AssetSlot::new(AssetPathType::Url(path.to_string_lossy().to_string())));
        }
        default_modal_action(action)
    }

    fn render(&self, settings: &SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        let mut slot = self.slot.borrow_mut();
        if let Some(i) = request_asset_image(graphics, &mut slot) {
            i.render(graphics, Rect::new(LogicalPosition::new(0., 0.), LogicalSize::new(100., 100.)))
        }

        self.list.render(settings, rect, graphics);
    }
}

pub fn on_game_found_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<ListModal<GameScrapeResult>>().unwrap();

        let item = content.get_selected();
        crate::platform::insert_game(state, &item.info, item.boxart.clone());
    }
}