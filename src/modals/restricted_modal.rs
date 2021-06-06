use druid_shell::kurbo::{Rect};
use druid_shell::piet::{Piet, RenderContext};
use crate::Actions;
use crate::colors::*;
use crate::modals::{ModalResult, ModalContent, DeferredModalAction};
use crate::restrictions::{RestrictedPasscode, PasscodeEquality, passcodes_equal};

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
    fn get_height(&self) -> f64 { crate::font::FONT_SIZE + crate::ui::MARGIN }

    fn action(&mut self, action: &Actions, _: &mut DeferredModalAction) -> ModalResult {
        let code = match action {
            Actions::Accept => return ModalResult::Ok,
            Actions::Back => return ModalResult::Cancel,
            Actions::KeyPress(code) => *code,
            _ => action_to_u32(action),
        };
        self.pass.add_digit(code);
        ModalResult::None
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, piet: &mut Piet) {
        let item_label = crate::widgets::get_drawable_text(piet, crate::font::FONT_SIZE, "*", get_font_color(settings));
        for i in 0..self.pass.len() {
            piet.draw_text(&item_label, (rect.x0 + i as f64 * crate::font::FONT_SIZE, rect.y0));

        }
    }
}

pub struct VerifyRestrictedModal { 
    pass: RestrictedPasscode,
    target: RestrictedPasscode,
    attempts: u8,
    request: fn(&mut dyn std::any::Any),
    data: *mut dyn std::any::Any,
}
impl VerifyRestrictedModal {
    pub fn new(target: RestrictedPasscode, action: fn(&mut dyn std::any::Any), data: *mut dyn std::any::Any) -> VerifyRestrictedModal {
        VerifyRestrictedModal {
            pass: RestrictedPasscode::default(),
            target: target,
            attempts: 0,
            request: action,
            data: data
        }
    }

    pub fn run_action(&self) {
        let data = unsafe { &mut *self.data };
        (self.request)(data);
    }
}

impl ModalContent for VerifyRestrictedModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_height(&self) -> f64 { crate::font::FONT_SIZE + crate::ui::MARGIN }

    fn action(&mut self, action: &Actions, _: &mut DeferredModalAction) -> ModalResult {
        //Get the key (or action which can be translated to a key)
        let code = match action {           
            Actions::Accept => return ModalResult::Ok,
            Actions::Back => return ModalResult::Cancel,
            Actions::KeyPress(code) => *code,
            _ => action_to_u32(action)
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

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, piet: &mut Piet) {
        let item_label = crate::widgets::get_drawable_text(piet, crate::font::FONT_SIZE, "*", get_font_color(settings));
        for i in 0..self.pass.len() {
            piet.draw_text(&item_label, (rect.x0 + i as f64 * crate::font::FONT_SIZE, rect.y0));
        }
    }
}

fn action_to_u32(action: &Actions) -> u32 {
    match action {
        Actions::Info => 1,
        Actions::Accept => 2,
        Actions::Select => 3,
        Actions::Back => 4,
        Actions::Up => 5,
        Actions::Down => 6,
        Actions::Left => 7,
        Actions::Right => 8,
        Actions::Filter => 9,
        _ => panic!("Invalid action converting to u32"),
    }
}