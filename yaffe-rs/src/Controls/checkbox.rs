use super::draw_label_and_box;
use crate::controls::LABEL_SIZE;
use crate::input::{Actions, InputType};
use crate::ui::{AnimationManager, LayoutElement, UiElement, ValueElement, WidgetId};
use crate::utils::Rect;
use crate::{Graphics, LogicalSize};
use winit::keyboard::KeyCode;

crate::widget!(
    pub struct CheckBox {
        checked: bool = false,
        label: String = String::new()
    }
);

impl CheckBox {
    pub fn from(label: String, checked: bool) -> CheckBox {
        let mut checkbox = CheckBox::new();
        checkbox.label = label;
        checkbox.checked = checked;
        checkbox
    }
}
impl<T: 'static, D: 'static> UiElement<T, D> for CheckBox {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize {
        LogicalSize::new(graphics.font_size() + LABEL_SIZE, graphics.font_size())
    }
    fn action(&mut self, _state: &mut T, _: &mut AnimationManager, action: &Actions, _handler: &mut D) -> bool {
        if let Actions::KeyPress(InputType::Key(KeyCode::Space, _, _)) = action {
            self.checked = !self.checked;
            return true;
        }
        false
    }

    fn render(&mut self, graphics: &mut Graphics, _: &T, current_focus: &WidgetId) {
        let control = draw_label_and_box(graphics, self.layout().top_left(), graphics.font_size(), &self.label);

        let base = graphics.accent_color();
        if self.get_id() == *current_focus {
            graphics.outline_rect(control, 2., base);
        }

        if self.checked {
            graphics.draw_rectangle(
                Rect::from_tuples(
                    (control.left() + 4., control.top() + 4.),
                    (control.right() - 4., control.bottom() - 4.),
                ),
                base,
            )
        }
    }
}
impl ValueElement<bool> for CheckBox {
    fn value(&self) -> bool { self.checked }
}
