use crate::controls::{PassBox, RestrictedPasscode};
use crate::modals::{ModalContentElement, ModalInputHandler};
use crate::ui::{ContainerSize, LayoutElement, UiContainer, ValueElement};
use crate::{DeferredAction, YaffeState};

pub enum RestrictedMode {
    On(RestrictedPasscode),
    Off,
}

pub struct SetRestrictedModal;

impl SetRestrictedModal {
    #[allow(clippy::new_ret_no_self)]
    pub fn new() -> ModalContentElement<YaffeState> {
        let mut modal = ModalContentElement::new(SetRestrictedModal, false);
        let pass = PassBox::new();
        let pass_id = pass.get_id();
        modal.add_child(pass, ContainerSize::Shrink);
        modal.focus(pass_id);
        modal
    }
}

impl ModalInputHandler<YaffeState> for SetRestrictedModal {
    fn as_any(&self) -> &dyn std::any::Any { self }

    fn on_close(
        &self,
        state: &mut YaffeState,
        result: bool,
        content: &UiContainer<YaffeState>,
        handler: &mut DeferredAction<YaffeState>,
    ) {
        if result {
            let content = crate::convert_to!(content.get_child(0), PassBox);
            let pass = content.value();

            match state.restricted_mode {
                RestrictedMode::On(p) => {
                    if pass == p {
                        state.restricted_mode = RestrictedMode::Off;
                    } else {
                        handler.display_toast("Incorrect passcode", 1.);
                    }
                }
                RestrictedMode::Off => state.restricted_mode = RestrictedMode::On(pass),
            }
        }
    }
}

pub fn verify_restricted_action(state: &mut YaffeState) -> bool {
    if let RestrictedMode::On(_) = state.restricted_mode {
        return false;
    }
    true
}
