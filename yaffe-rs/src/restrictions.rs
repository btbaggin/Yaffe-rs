use crate::{YaffeState, modals::ModalContent, modals::ModalResult, modals::SetRestrictedModal, modals::VerifyRestrictedModal };

pub const MAX_ATTEMPTS: u8 = 3;
const DISABLE_TAG: &'static str = "disable";
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
    Pending,
}

#[repr(u8)]
pub enum PasscodeEquality {
    Lengths,
    Total,
    None,
}
pub fn passcodes_equal(source: &RestrictedPasscode, target: &RestrictedPasscode) -> PasscodeEquality {
    if source.len() == target.len() {
        for i in 0..source.len() {
            if source.code[i] != target.code[i] { return PasscodeEquality::Lengths; }
        }
        return PasscodeEquality::Total;
    }
    PasscodeEquality::None
}

pub fn on_restricted_modal_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<SetRestrictedModal>().unwrap();
        let pass = content.get_passcode();

        state.restricted_mode = RestrictedMode::On(pass);
    } else {
        state.restricted_mode = RestrictedMode::Off;
    }
}

fn on_verify_restricted_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, handler: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<VerifyRestrictedModal>().unwrap();

        handler.finalize_restricted_action(content.tag());
        state.restricted_last_approve = Some(std::time::Instant::now());
    }
}

pub fn try_disable_restrictions(state: &mut YaffeState, tag: &'static str) -> bool {
    if tag == DISABLE_TAG {
        state.restricted_mode = RestrictedMode::Off;
        return true;
    }
    false
}

pub fn disable_restrictions(state: &mut YaffeState, handler: &mut crate::DeferredAction) {
    verify_restricted_action(state, DISABLE_TAG, handler)
}

pub fn verify_restricted_action(state: &mut YaffeState, tag: &'static str, handler: &mut crate::DeferredAction) {
    if let RestrictedMode::On(pass) = state.restricted_mode {
        //Only as for approval if its past the last approval is no longer valid
        let approve = match state.restricted_last_approve {
            Some(t) => t.elapsed().as_secs() > state.settings.get_i32(crate::SettingNames::RestrictedApprovalValidTime) as u64,
            None => true,
        };
        
        if approve {
            let content = VerifyRestrictedModal::new(pass, tag);
            let content = Box::new(content);
            crate::modals::display_modal(state, "Verify actions", None, content, crate::modals::ModalSize::Third, Some(on_verify_restricted_close));
            return;
        } 
    } 

    handler.finalize_restricted_action(tag);
}