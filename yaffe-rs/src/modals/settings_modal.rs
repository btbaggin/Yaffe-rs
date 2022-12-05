use crate::{YaffeState, Actions, Rect};
use crate::modals::*;
use crate::ui_control::*;
use crate::logger::{UserMessage, LogEntry};
use crate::settings::{SettingsFile, SettingValue};
use std::str::FromStr;

const STARTUP_TASK: &str = "Yaffe";

pub struct SettingsModal {
    settings: FocusGroup<dyn UiControl>,
}
impl SettingsModal {
    pub fn new(settings: &SettingsFile) -> SettingsModal {
        let mut controls: FocusGroup<dyn UiControl> = FocusGroup::new();
        let mut setting_names = settings.get_full_settings();
        let set = crate::platform_layer::get_run_at_startup(STARTUP_TASK).log("Unable to get if Yaffe runs at startup");
        controls.insert("run_at_startup", Box::new(CheckBox::new(set)));

        setting_names.sort_by(|x, y| x.0.cmp(&y.0));
        for (name, default) in setting_names {
            let control = match default {
                SettingValue::Color(c) => TextBox::new(rgba_string(&c)),
                SettingValue::F32(f) => TextBox::new(f.to_string()),
                SettingValue::I32(i) => TextBox::new(i.to_string()),
                SettingValue::String(s) => TextBox::new(s.clone()),
            };
            controls.insert(&name, Box::new(control));
        }

        SettingsModal { settings: controls }
    }
}

impl ModalContent for SettingsModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &crate::Graphics) -> LogicalSize {
        let height = (get_font_size(settings, graphics) + MARGIN) * self.settings.len() as f32;
        LogicalSize::new(modal_width(rect, ModalSize::Half), height)
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        let mut y = rect.top();
        let font_size = get_font_size(settings, graphics);
        for (k, v) in &self.settings {
            let rect = Rect::from_tuples((rect.left(), y), (rect.right(), y + font_size));
            v.render(graphics, settings, &rect, &k, self.settings.is_focused(&v));
            y += font_size + MARGIN;
        }
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        if !self.settings.action(action) {
            if let Some(focus) = self.settings.focus() {
                focus.action(action);
            }
        }
        default_modal_action(action)
    }
}

pub fn on_settings_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<SettingsModal>().unwrap();

        //Update settings values
        for (name, control) in &content.settings {
            match &name[..] {
                "run_at_startup" => {
                    let value = bool::from_str(control.value()).unwrap();
                    crate::platform_layer::set_run_at_startup(STARTUP_TASK, value).display_failure("Unable to save settings", state)
                },
                "logging_level" => {
                    crate::logger::set_log_level(control.value());
                    state.settings.set_setting(&name, control.value()).display_failure("Unable to save settings", state)
                },
                _ => state.settings.set_setting(&name, control.value()).display_failure("Unable to save settings", state),
            };
        }

        //Save settings
        state.settings.serialize().display_failure("Unable to save settings", state);
    }
}