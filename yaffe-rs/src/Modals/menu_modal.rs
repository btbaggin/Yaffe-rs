use crate::controls::List;
use crate::logger::UserMessage;
use crate::modals::{
    DisplayModal, ModalContentElement, ModalInputHandler, ModalSize, PlatformDetailModal, SetRestrictedModal,
    SettingsModal,
};
use crate::ui::{ContainerSize, UiContainer};
use crate::{DeferredAction, YaffeState};

pub struct MenuModal;

impl MenuModal {
    pub fn from(items: Vec<String>) -> ModalContentElement<crate::YaffeState> {
        let mut modal = ModalContentElement::new(MenuModal, false);
        let list = List::from(items);
        modal.add_child(list, ContainerSize::Shrink);
        modal
    }
}

impl ModalInputHandler<YaffeState> for MenuModal {
    fn as_any(&self) -> &dyn std::any::Any { self }

    fn on_close(
        &self,
        state: &mut YaffeState,
        result: bool,
        content: &UiContainer<YaffeState>,
        handler: &mut DeferredAction<YaffeState>,
    ) {
        if result {
            let elements = crate::convert_to!(content.get_child(0), crate::controls::List<String>);
            let selected = elements.get_selected().as_str();

            match selected {
                "Add Emulator" => {
                    let content = PlatformDetailModal::emulator();
                    handler.display_modal(DisplayModal::new(
                        "New Emulator",
                        Some("Confirm"),
                        content,
                        ModalSize::Third,
                    ));
                }
                "Settings" => {
                    let content = SettingsModal::from(&state.settings);
                    handler.display_modal(DisplayModal::new("Settings", Some("Confirm"), content, ModalSize::Third));
                }
                "Disable Restricted Mode" | "Enable Restricted Mode" => {
                    let content = SetRestrictedModal::new();
                    handler.display_modal(DisplayModal::new(
                        "Restricted Mode",
                        Some("Set passcode"),
                        content,
                        ModalSize::Third,
                    ));
                }
                "Scan For New Roms" => crate::platform::scan_new_files(state, handler),
                "Exit Yaffe" => state.exit(),
                "Shut Down" => {
                    if crate::os::shutdown().display_failure("Failed to shut down", handler).is_some() {
                        state.exit();
                    }
                }
                _ => panic!("Unknown menu option"),
            }
        }
    }
}
