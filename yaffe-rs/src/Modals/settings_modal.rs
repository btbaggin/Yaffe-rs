use crate::logger::{LogEntry, UserMessage};
use crate::modals::*;
use crate::settings::SettingsFile;
use crate::ui::{CheckBox, Container, Control, TextBox};
use crate::{Actions, Rect, YaffeState};
use std::str::FromStr;

const STARTUP_TASK: &str = "Yaffe";

pub struct SettingsModal {
    names: Vec<String>,
    settings: Container,
}
impl SettingsModal {
    pub fn new(settings: &SettingsFile) -> SettingsModal {
        let mut controls = Container::vertical(1.);
        let mut setting_names = settings.get_full_settings();
        let set = crate::os::get_run_at_startup(STARTUP_TASK).log("Unable to get if Yaffe runs at startup");
        controls.add_field("run_at_startup", CheckBox::new("run_at_startup".to_string(), set));

        let mut names = vec![];
        names.push("run_at_startup".to_string());

        setting_names.sort_by(|x, y| x.0.cmp(&y.0));
        for (name, default) in setting_names {
            names.push(name.clone());

            let value = default.to_string();
            controls.add_field(&name, TextBox::new(name.clone(), value));
        }

        SettingsModal { names, settings: controls }
    }
}

impl ModalContent for SettingsModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, rect: Rect, graphics: &crate::Graphics) -> LogicalSize {
        let height = (graphics.font_size() + MARGIN) * self.settings.child_count() as f32;
        LogicalSize::new(Self::modal_width(rect, ModalSize::Half), height)
    }

    fn render(&self, rect: Rect, graphics: &mut crate::Graphics) { self.settings.render(graphics, &rect); }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        self.settings.action(action);
        Self::default_modal_action(action)
    }
}

pub fn on_settings_close(
    state: &mut YaffeState,
    result: ModalResult,
    content: &dyn ModalContent,
    _: &mut crate::DeferredAction,
) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<SettingsModal>().unwrap();

        //Update settings values
        for name in &content.names {
            let control = content.settings.by_tag(name).unwrap();

            match &name[..] {
                "run_at_startup" => {
                    let value = bool::from_str(control.value()).unwrap();
                    crate::os::set_run_at_startup(STARTUP_TASK, value).display_failure("Unable to save settings", state)
                }
                "logging_level" => {
                    crate::logger::set_log_level(control.value());
                    state.settings.set_setting(name, control.value()).display_failure("Unable to save settings", state)
                }
                _ => {
                    state.settings.set_setting(name, control.value()).display_failure("Unable to save settings", state)
                }
            };
        }

        //Save settings
        state.settings.serialize().display_failure("Unable to save settings", state);
    }
}
