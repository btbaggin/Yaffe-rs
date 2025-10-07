use crate::controls::{PassBox, RestrictedPasscode};
use crate::modals::{ModalContent, ModalContentElement};
use crate::ui::{ContainerSize, LayoutElement, ValueElement};
use crate::YaffeState;

pub enum RestrictedMode {
    On(RestrictedPasscode),
    Off,
}

pub struct SetRestrictedModal;

impl SetRestrictedModal {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> ModalContentElement {
        let mut modal = ModalContentElement::new(SetRestrictedModal, false);
        let pass = PassBox::new();
        let pass_id = pass.get_id();
        modal.add_child(pass, ContainerSize::Shrink);
        modal.focus = Some(pass_id);
        modal
    }
}

impl ModalContent for SetRestrictedModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

pub fn on_restricted_modal_close(state: &mut YaffeState, result: bool, content: &ModalContentElement) {
    if result {
        let content = crate::convert_to!(content.get_child(0), PassBox);
        let pass = content.value();

        match state.restricted_mode {
            RestrictedMode::On(p) => {
                if pass == p {
                    state.restricted_mode = RestrictedMode::Off;
                } else {
                    // state.display_toast(0, );
                    // TODO toast?
                }
            }
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
