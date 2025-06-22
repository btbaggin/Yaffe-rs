use crate::modals::{ModalContent, ModalResult};
use crate::restrictions::RestrictedPasscode;
use crate::ui::{ModalSize, MARGIN};
use crate::{Actions, LogicalPosition, LogicalSize, Rect};
use std::hash::{Hash, Hasher};

pub struct SetRestrictedModal {
    pass: RestrictedPasscode,
}
impl SetRestrictedModal {
    pub fn new() -> SetRestrictedModal { SetRestrictedModal { pass: RestrictedPasscode::default() } }

    pub fn get_passcode(&self) -> RestrictedPasscode { self.pass }
}

impl ModalContent for SetRestrictedModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, rect: Rect, graphics: &crate::Graphics) -> LogicalSize {
        let height = graphics.font_size() + MARGIN;
        LogicalSize::new(Self::modal_width(rect, ModalSize::Third), height)
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        let code = match action {
            Actions::Accept => return ModalResult::Ok,
            Actions::Back => return ModalResult::Cancel,
            Actions::KeyPress(crate::input::InputType::Key(code, _, _)) => *code as u8 as char,
            _ => action_to_char(action),
        };
        self.pass.add_digit(code);
        ModalResult::None
    }

    fn render(&self, rect: Rect, graphics: &mut crate::Graphics) {
        let font_size = graphics.font_size();

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
