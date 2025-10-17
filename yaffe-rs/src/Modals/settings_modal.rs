use crate::controls::{CheckBox, TextBox};
use crate::logger::{LogEntry, UserMessage};
use crate::modals::{ModalContentElement, ModalInputHandler};
use crate::settings::SettingsFile;
use crate::ui::{ContainerSize, UiContainer, ValueElement};
use crate::{DeferredAction, YaffeState};

const STARTUP_TASK: &str = "Yaffe";

pub struct SettingsModal {
    names: Vec<String>,
}

impl SettingsModal {
    pub fn from(settings: &SettingsFile) -> ModalContentElement<YaffeState> {
        let mut setting_names = settings.get_full_settings();
        setting_names.sort_by(|x, y| x.0.cmp(&y.0));

        let mut all_settings = vec!["run_at_startup".to_string()];
        for (s, _) in &setting_names {
            all_settings.push(s.clone());
        }
        let content = SettingsModal { names: all_settings };

        let mut modal = ModalContentElement::new(content, true);
        let set = crate::os::get_run_at_startup(STARTUP_TASK).log("Unable to get if Yaffe runs at startup");
        modal.add_child(CheckBox::from("run_at_startup".to_string(), set), ContainerSize::Shrink);

        for (name, default) in setting_names {
            let element = TextBox::from(&name.clone(), &default.to_string());
            modal.add_child(element, ContainerSize::Shrink);
        }
        modal.focus_first();
        modal
    }
}

impl ModalInputHandler<YaffeState> for SettingsModal {
    fn as_any(&self) -> &dyn std::any::Any { self }

    fn on_close(
        &self,
        state: &mut YaffeState,
        result: bool,
        content: &UiContainer<YaffeState>,
        handler: &mut DeferredAction<YaffeState>,
    ) {
        if result {
            for (i, name) in self.names.iter().enumerate() {
                match name.as_str() {
                    "run_at_startup" => {
                        let run_at_startup = crate::convert_to!(content.get_child(i), CheckBox);
                        crate::os::set_run_at_startup(STARTUP_TASK, run_at_startup.value())
                            .display_failure("Unable to save settings", handler);
                    }
                    _ => {
                        let control = crate::convert_to!(content.get_child(i), TextBox);
                        state
                            .settings
                            .set_setting(name, &control.value())
                            .display_failure("Unable to save settings", handler);
                    }
                };
            }

            //Save settings
            state.settings.serialize().display_failure("Unable to save settings", handler);
        }
    }
}
