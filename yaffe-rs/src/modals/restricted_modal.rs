use crate::{Actions, LogicalPosition, LogicalSize, Rect};
use crate::modals::{ModalResult, ModalContent};
use crate::restrictions::RestrictedPasscode;
use crate::ui_control::{get_font_color, get_font_size, MARGIN};
use crate::settings::SettingsFile;
use super::{modal_width, ModalSize};
use std::hash::{Hash, Hasher};

pub struct SetRestrictedModal {
    pass: RestrictedPasscode,
}
impl SetRestrictedModal {
    pub fn new() -> SetRestrictedModal {
        SetRestrictedModal { pass: RestrictedPasscode::default(), }
    }

    pub fn get_passcode(&self) -> RestrictedPasscode {
        self.pass
    }
}

impl ModalContent for SetRestrictedModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, settings: &SettingsFile, rect: Rect, graphics: &crate::Graphics) -> LogicalSize { 
        let height = get_font_size(settings, graphics) + MARGIN;
        LogicalSize::new(modal_width(rect, ModalSize::Third), height)
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        let code = match action {
            Actions::Accept => return ModalResult::Ok,
            Actions::Back => return ModalResult::Cancel,
            Actions::KeyPress(crate::input::InputType::Char(code)) => *code,
            _ => action_to_char(action)
        };
        self.pass.add_digit(code);
        ModalResult::None
    }

    fn render(&self, settings: &SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        let font_size = get_font_size(settings, graphics);

        let item_label = crate::widgets::get_drawable_text(font_size, "*");
        for i in 0..self.pass.len() {
            graphics.draw_text(LogicalPosition::new(rect.left() + i as f32 * font_size, rect.top()), get_font_color(settings), &item_label);
        }
    }
}

fn action_to_char(action: &Actions) -> char {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    action.hash(&mut hasher);
    hasher.finish() as u8 as char
}