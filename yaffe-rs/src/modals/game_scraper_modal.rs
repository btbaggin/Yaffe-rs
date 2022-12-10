
use crate::ui::{get_drawable_text_with_wrap, ModalResult, ModalContent, ModalSize};
use crate::{YaffeState, Rect, LogicalSize, LogicalPosition};
use crate::scraper::GameScrapeResult;
use crate::settings::SettingsFile;
use crate::input::Actions;
use crate::assets::{AssetSlot, AssetPathType, request_asset_image};
use std::cell::RefCell;
use crate::ui::{List, get_font_color, get_font_size, MARGIN};


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
        LogicalSize::new(Self::modal_width(rect, ModalSize::Full), rect.height())
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        if self.list.update(action) {
            let item = self.list.get_selected();
            self.slot = RefCell::new(AssetSlot::new(AssetPathType::Url(format!("https://cdn.thegamesdb.net/images/medium/{}", item.boxart))));
        }
        Self::default_modal_action(action)
    }

    fn render(&self, settings: &SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        let half_size = LogicalSize::new(rect.width() / 2., rect.height());
        let left = Rect::point_and_size(*rect.top_left(), half_size);
        
        let image_container = Rect::percent(left, LogicalSize::new(0.75, 0.25));

        let mut slot = self.slot.borrow_mut();
        let size = crate::ui::image_fill(graphics, &mut slot, &image_container.size(), false);
        if let Some(i) = request_asset_image(graphics, &mut slot) {
            i.render(graphics, Rect::point_and_size(*left.top_left(), size))
        }

        let item = self.list.get_selected();
        graphics.simple_text(LogicalPosition::new(left.left() + size.x + MARGIN, left.top()), settings, &format!("Players: {}", item.info.players));

        let text = get_drawable_text_with_wrap(get_font_size(settings, graphics), &item.info.overview, left.width());
        graphics.draw_text(LogicalPosition::new(left.left(), left.top() + size.y), get_font_color(settings), &text);
        
        let right = Rect::point_and_size(LogicalPosition::new(rect.left() + half_size.x, rect.top()), half_size);
        self.list.render(settings, right, graphics);
    }
}

pub fn on_game_found_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<GameScraperModal>().unwrap();

        let item = content.list.get_selected();
        crate::platform::insert_game(state, &item.info, item.boxart.clone());
    }
}