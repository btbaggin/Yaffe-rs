use crate::{Actions, LogicalPosition, Rect, colors::*, ui::*};
use crate::modals::{ModalResult, ModalContent};
use crate::restrictions::{RestrictedPasscode, PasscodeEquality, passcodes_equal};
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
    fn get_height(&self, settings: &crate::settings::SettingsFile, graphics: &crate::Graphics, _: f32) -> f32 { 
        crate::font::get_font_size(settings, graphics) + MARGIN
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

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        let font_size = crate::font::get_font_size(settings, graphics);

        let item_label = crate::widgets::get_drawable_text(font_size, "*");
        for i in 0..self.pass.len() {
            graphics.draw_text(LogicalPosition::new(rect.left() + i as f32 * font_size, rect.top()), get_font_color(settings), &item_label);
        }
    }
}

pub struct VerifyRestrictedModal { 
    pass: RestrictedPasscode,
    target: RestrictedPasscode,
    attempts: u8,
    tag: &'static str,
}
impl VerifyRestrictedModal {
    pub fn new(target: RestrictedPasscode, tag: &'static str) -> VerifyRestrictedModal {
        VerifyRestrictedModal {
            pass: RestrictedPasscode::default(),
            target: target,
            attempts: 0,
            tag,
        }
    }

    pub fn tag(&self) -> &'static str {
        self.tag
    }
}

impl ModalContent for VerifyRestrictedModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_height(&self, settings: &crate::settings::SettingsFile, graphics: &crate::Graphics, _: f32) -> f32 { 
        crate::font::get_font_size(settings, graphics) + MARGIN 
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        //Get the key (or action which can be translated to a key)
        let code = match action {           
            Actions::Accept => return ModalResult::Ok,
            Actions::Back => return ModalResult::Cancel,
            Actions::KeyPress(crate::input::InputType::Char(code)) => *code,
            _ => action_to_char(action),
        };

        self.pass.add_digit(code);             
        match passcodes_equal(&self.pass, &self.target) {
            PasscodeEquality::None => {},
            PasscodeEquality::Lengths => {
                self.attempts += 1;
                if self.attempts > crate::restrictions::MAX_ATTEMPTS { return ModalResult::Cancel; }
                else { self.pass = RestrictedPasscode::default(); }
            }
            PasscodeEquality::Total => return ModalResult::Ok,
        }
        ModalResult::None
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        let font_size = crate::font::get_font_size(settings, graphics);
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