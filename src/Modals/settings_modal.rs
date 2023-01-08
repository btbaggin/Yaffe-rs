use crate::{YaffeState, Actions, Rect};
use crate::modals::*;
use crate::ui::{Container, TextBox, Control, rgba_string, CheckBox};
use crate::logger::{UserMessage, LogEntry};
use crate::settings::{SettingsFile, SettingValue};
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
        let set = crate::platform_layer::get_run_at_startup(STARTUP_TASK).log("Unable to get if Yaffe runs at startup");
        controls.add_field("run_at_startup", CheckBox::new("run_at_startup".to_string(), set));

        let mut names = vec!();
        names.push("run_at_startup".to_string());

        setting_names.sort_by(|x, y| x.0.cmp(&y.0));
        for (name, default) in setting_names {
            names.push(name.clone());

            let value = match default {
                SettingValue::Color(c) => rgba_string(&c),
                SettingValue::F32(f) => f.to_string(),
                SettingValue::I32(i) => i.to_string(),
                SettingValue::String(s) => s.clone(),
            };
            controls.add_field(&name, TextBox::new(name.clone(), value));
        }

        SettingsModal { names, settings: controls }
    }
}

impl ModalContent for SettingsModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &crate::Graphics) -> LogicalSize {
        let height = (get_font_size(settings, graphics) + MARGIN) * self.settings.child_count() as f32;
        LogicalSize::new(Self::modal_width(rect, ModalSize::Half), height)
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        self.settings.render(graphics, settings, &rect);
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        self.settings.action(action);
        Self::default_modal_action(action)
    }
}

pub fn on_settings_close(state: &mut YaffeState, result: ModalResult, content: &dyn ModalContent, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<SettingsModal>().unwrap();

        //Update settings values
        for name in &content.names {
            let control = content.settings.by_tag(name).unwrap();

            match &name[..] {
                "run_at_startup" => {
                    let value = bool::from_str(control.value()).unwrap();
                    crate::platform_layer::set_run_at_startup(STARTUP_TASK, value).display_failure("Unable to save settings", state)
                },
                "logging_level" => {
                    crate::logger::set_log_level(control.value());
                    state.settings.set_setting(name, control.value()).display_failure("Unable to save settings", state)
                },
                _ => state.settings.set_setting(name, control.value()).display_failure("Unable to save settings", state),
            };
        }

        //Save settings
        state.settings.serialize().display_failure("Unable to save settings", state);
    }
}