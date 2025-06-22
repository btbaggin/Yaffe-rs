mod game_scraper_modal;
mod list_modal;
mod platform_detail_modal;
mod platform_scraper_modal;
mod restricted_modal;
mod settings_modal;

use crate::ui::MARGIN;
use crate::ui::{ModalContent, ModalResult, ModalSize};
use crate::{LogicalSize, Rect};

pub use game_scraper_modal::{on_game_found_close, GameScraperModal};
pub use list_modal::ListModal;
pub use platform_detail_modal::{on_add_platform_close, on_update_platform_close, PlatformDetailModal};
pub use platform_scraper_modal::{on_platform_found_close, PlatformScraperModal};
pub use restricted_modal::SetRestrictedModal;
pub use settings_modal::{on_settings_close, SettingsModal};

//Modal for displaying a simple string
pub struct MessageModalContent {
    message: String,
}
impl MessageModalContent {
    pub fn new(message: &str) -> MessageModalContent { MessageModalContent { message: String::from(message) } }
}
impl ModalContent for MessageModalContent {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, rect: Rect, graphics: &crate::Graphics) -> LogicalSize {
        let width = Self::modal_width(rect, ModalSize::Half);
        LogicalSize::new(width, graphics.font_size() + crate::ui::MARGIN)
    }

    fn action(&mut self, action: &crate::Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        Self::default_modal_action(action)
    }

    fn render(&self, rect: Rect, graphics: &mut crate::Graphics) {
        let name_label = crate::ui::get_drawable_text_with_wrap(
            graphics,
            graphics.font_size(),
            &self.message,
            rect.width() * graphics.scale_factor,
        );
        graphics.draw_text(*rect.top_left(), graphics.font_color(), &name_label);
    }
}
