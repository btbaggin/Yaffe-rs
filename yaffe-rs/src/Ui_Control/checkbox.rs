use super::{UiControl, draw_label_and_box, get_accent_color, get_font_size};
use crate::Actions;
use crate::settings::SettingsFile;
use crate::input::InputType;
use crate::utils::Rect;
use glutin::event::VirtualKeyCode;

pub struct CheckBox {
    checked: bool,
}
impl CheckBox {
    pub fn new(checked: bool) -> CheckBox {
        CheckBox { checked }
    }
}
impl UiControl for CheckBox {
    fn render(&self, graphics: &mut crate::Graphics, settings: &SettingsFile, container: &Rect, label: &str, focused: bool) {
        let control = draw_label_and_box(graphics, settings, &container.top_left(), get_font_size(settings, graphics), label, focused);

        if self.checked {
            let base = get_accent_color(settings);
            graphics.draw_rectangle(Rect::from_tuples((control.left() + 4., control.top() + 4.), (control.right() - 4., control.bottom() - 4.)), base)
        }
    }

    fn value(&self) -> &str {
        if self.checked { "true" } else { "false" }
    }

    fn action(&mut self, action: &Actions) {
        if let Actions::KeyPress(InputType::Key(VirtualKeyCode::Space)) = action {
            self.checked = !self.checked;
        }
    }
}