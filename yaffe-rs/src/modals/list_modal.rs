use crate::{Rect, LogicalSize, Actions};
use crate::modals::{ModalResult, ModalContent, default_modal_action, modal_width, ModalSize};
use crate::ui_control::{List, ListItem, get_font_size};
use crate::settings::SettingsFile;

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
    fn size(&self, settings: &SettingsFile, rect: Rect, graphics: &crate::Graphics) -> LogicalSize { 
        let count = self.list.items.len();
        let height = count as f32 * get_font_size(settings, graphics);

        LogicalSize::new(modal_width(rect, ModalSize::Third), height)
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        self.list.update(action);
        default_modal_action(action)
    }

    fn render(&self, settings: &SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        self.list.render(settings, rect, graphics)
    }
}