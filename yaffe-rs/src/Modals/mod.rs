mod game_scraper_modal;
mod list_modal;
mod platform_detail_modal;
mod platform_scraper_modal;
mod restricted_modal;
mod settings_modal;

use crate::ui::{UiElement, WidgetId, AnimationManager, LayoutElement, ModalAction};
use crate::input::{Actions};
use crate::Graphics;

pub use game_scraper_modal::{on_game_found_close, GameScraperModal};
pub use list_modal::ListModal;
pub use platform_detail_modal::{on_add_platform_close, on_update_platform_close, PlatformDetailModal};
pub use platform_scraper_modal::{on_platform_found_close, PlatformScraperModal};
pub use restricted_modal::SetRestrictedModal;
pub use settings_modal::{on_settings_close, SettingsModal};

//Modal for displaying a simple string
crate::widget!(
    pub struct MessageModalContent {
        message: String = String::new()
    }
);

impl MessageModalContent {
    pub fn from(message: &str) -> MessageModalContent {
        let mut content = MessageModalContent::new();
        content.message = message.to_string();
        content
    }
}
impl UiElement<(), ModalAction> for MessageModalContent {
    // fn as_any(&self) -> &dyn std::any::Any { self }
    // fn size(&self, rect: Rect, graphics: &crate::Graphics) -> LogicalSize {
    //     let width = Self::modal_width(rect, ModalSize::Half);
    //     let rows = self.message.len() as f32 / 80.;
    //     LogicalSize::new(width, (graphics.font_size() * rows) + crate::ui::MARGIN)
    // }

    fn action(&mut self, _state: &mut (), _: &mut AnimationManager, action: &Actions, handler: &mut ModalAction) -> bool {
        handler.close_if_accept(action)
    }

    fn render(&mut self, graphics: &mut Graphics, _: &(), _: &WidgetId) {
        let rect = self.layout();
        let name_label = crate::ui::get_drawable_text_with_wrap(
            graphics,
            graphics.font_size(),
            &self.message,
            rect.width() * graphics.scale_factor,
        );
        self.size.y = name_label.height();
        graphics.draw_text(*rect.top_left(), graphics.font_color(), &name_label);
    }
}
