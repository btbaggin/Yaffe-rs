use crate::{Rect, LogicalSize, Actions};
use crate::modals::{ModalResult, ModalContent, default_modal_action, modal_width, ModalSize};
use crate::ui_control::{List, ListItem};

/// Allow displaying a list of items that can be selected
/// Items must implement `ListItem` trait
pub struct ListModal<T: ListItem> {
    list: List<T>,
}
impl<T: ListItem> ListModal<T> {
    pub fn new(items: Vec<T>) -> ListModal<T> {
        ListModal { list: List::new(items) }
    }

    pub fn get_selected(&self) -> &T {
        self.list.get_selected()
    }
}

impl<T: 'static + ListItem> ModalContent for ListModal<T> {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &crate::Graphics) -> LogicalSize { 
        let count = self.list.items.len();
        let height = count as f32 * crate::font::get_font_size(settings, graphics);

        LogicalSize::new(modal_width(rect, ModalSize::Third), height)
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        self.list.update(action);
        default_modal_action(action)
        // match action {
        //     Actions::Down => {
        //         if self.index < self.items.len() - 1 { self.index += 1; }
        //         ModalResult::None
        //     }
        //     Actions::Up => {
        //         if self.index > 0 { self.index -= 1; }
        //         ModalResult::None
        //     }
        //     _ => default_modal_action(action)
        // }
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        self.list.render(settings, rect, graphics)
        // let mut pos = *rect.top_left();
        // let font_size = crate::font::get_font_size(settings, graphics);

        // //Title
        // if let Some(t) = &self.title {
        //     let title_label = crate::widgets::get_drawable_text(font_size, &t);
        //     graphics.draw_text(pos, get_font_color(settings), &title_label);
        //     pos.y += font_size;
        // }

        // //Item list
        // for (i, item) in self.items.iter().enumerate() {
        //     let display = item.to_display();

        //     if self.index == i {
        //         let rect = Rect::point_and_size(pos, LogicalSize::new(rect.width(), font_size));
        //         graphics.draw_rectangle(rect, get_accent_color(settings));
        //     }

        //     let item_label = crate::widgets::get_drawable_text(font_size, &display);
        //     graphics.draw_text(pos, get_font_color(settings), &item_label);
        //     pos.y += font_size;
        // }
    }
}