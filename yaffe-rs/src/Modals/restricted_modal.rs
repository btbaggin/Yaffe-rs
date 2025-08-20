use crate::input::{Actions, InputType};
use crate::restrictions::RestrictedPasscode;
use crate::ui::{UiElement, WidgetId, AnimationManager, LayoutElement};
use crate::{LogicalPosition, Graphics};
use std::hash::{Hash, Hasher};

crate::widget!(
    pub struct SetRestrictedModal {
        pass: RestrictedPasscode = RestrictedPasscode::default()
    }
);
impl SetRestrictedModal {
    // pub fn new() -> SetRestrictedModal { SetRestrictedModal { pass: RestrictedPasscode::default() } }

    pub fn get_passcode(&self) -> RestrictedPasscode { self.pass }
}

impl<T: 'static, D: 'static> UiElement<T, D> for SetRestrictedModal {
    // fn size(&self, rect: Rect, graphics: &crate::Graphics) -> LogicalSize {
    //     let height = graphics.font_size() + MARGIN;
    //     LogicalSize::new(Self::modal_width(rect, ModalSize::Third), height)
    // }

    fn action(&mut self, _state: &mut T, _: &mut AnimationManager, action: &Actions, _: &mut D) -> bool {
        let code = match action {
            Actions::Accept => return true,
            Actions::Back => return true,
            Actions::KeyPress(InputType::Key(code, _, _)) => *code as u8 as char,
            Actions::KeyPress(InputType::Gamepad(g)) => *g as u8 as char,
            _ => action_to_char(action),
        };
        self.pass.add_digit(code);
        false
    }

    fn render(&mut self, graphics: &mut Graphics, _: &T, _: &WidgetId) {
        let font_size = graphics.font_size();
        let rect = self.layout();

        let item_label = crate::ui::get_drawable_text(graphics, font_size, "*");
        for i in 0..self.pass.len() {
            graphics.draw_text(
                LogicalPosition::new(rect.left() + i as f32 * font_size, rect.top()),
                graphics.font_color(),
                &item_label,
            );
        }
    }
}

fn action_to_char(action: &Actions) -> char {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    action.hash(&mut hasher);
    hasher.finish() as u8 as char
}
