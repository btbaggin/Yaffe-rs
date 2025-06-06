use crate::{YaffeState, ui::ModalContent, ui::ModalResult, modals::SetRestrictedModal};

const PIN_SIZE: usize = 8;

#[derive(Default, Copy, Clone)]
pub struct RestrictedPasscode {
    code: [char; PIN_SIZE],
    length: usize,
}
impl RestrictedPasscode {
    pub fn add_digit(&mut self, code: char) {
        if self.length < PIN_SIZE {
            self.code[self.length] = code;
            self.length += 1;
        }
    }
    pub fn len(&self) -> usize {
        self.length
    }
}

pub enum RestrictedMode {
    On(RestrictedPasscode),
    Off,
}

fn passcodes_equal(source: &RestrictedPasscode, target: &RestrictedPasscode) -> bool {
    for i in 0..source.len() {
        if source.code[i] != target.code[i] { return false; }
    }
    true
}

pub fn on_restricted_modal_close(state: &mut YaffeState, result: ModalResult, content: &dyn ModalContent, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<SetRestrictedModal>().unwrap();
        let pass = content.get_passcode();

        match state.restricted_mode {
            RestrictedMode::On(p) => {
                if passcodes_equal(&pass, &p) {
                    state.restricted_mode = RestrictedMode::Off;
                }
            },
            RestrictedMode::Off => state.restricted_mode = RestrictedMode::On(pass),
        }
    }
}

pub fn verify_restricted_action(state: &mut YaffeState) -> bool {
    if let RestrictedMode::On(_) = state.restricted_mode {
        return false;
    } 
    true
}
