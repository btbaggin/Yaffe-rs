use super::{InputControl, Control, draw_label_and_box};
use crate::input::{InputType, Actions};
use crate::utils::Rect;
use glutin::event::VirtualKeyCode;

pub struct CheckBox {
    checked: bool,
    label: String,
    focused: bool,
}
impl CheckBox {
    pub fn new(label: String, checked: bool) -> CheckBox {
        CheckBox { label, checked, focused: false }
    }
}
impl Control for CheckBox {
    fn render(&self, graphics: &mut crate::Graphics, container: &Rect) -> crate::LogicalSize {
        let control = draw_label_and_box(graphics, container.top_left(), graphics.font_size(), &self.label);

        if self.checked {
            let base = graphics.accent_color();
            graphics.draw_rectangle(Rect::from_tuples((control.left() + 4., control.top() + 4.), (control.right() - 4., control.bottom() - 4.)), base)
        }

        crate::LogicalSize::new(control.width() + crate::ui::LABEL_SIZE, control.height())
    }

    fn action(&mut self, action: &Actions) {
        if let Actions::KeyPress(InputType::Key(VirtualKeyCode::Space)) = action {
            self.checked = !self.checked;
        }
    }

}
impl InputControl for CheckBox {
    fn value(&self) -> &str { if self.checked { "true" } else { "false" } }
    fn set_focused(&mut self, value: bool) { self.focused = value; }
}