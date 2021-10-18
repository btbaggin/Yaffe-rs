use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::{font::*, ui::*, YaffeState, Actions};
use crate::modals::*;
use crate::controls::*;
use crate::logger::{UserMessage, LogTypes};
use crate::settings::{SettingsFile, SettingValue};
use std::str::FromStr;

const STARTUP_TASK: &str = "Yaffe";

pub struct SettingsModal {
    settings: FocusGroup<dyn UiControl>,
}
impl SettingsModal {
    pub fn new(settings: &SettingsFile, plugin: Option<&std::cell::RefCell<crate::plugins::Plugin>>) -> SettingsModal {
        let mut controls: FocusGroup<dyn UiControl> = FocusGroup::new();
        let setting_names = settings.get_full_settings(plugin);
        if let None = plugin {
            let set = match crate::platform_layer::get_run_at_startup(STARTUP_TASK) {
                Ok(v) => v,
                Err(e) => {
                    crate::logger::log_entry_with_message(LogTypes::Error, e, "Unable to get if Yaffe runs at startup");
                    false
                }
            };
            controls.insert("run_at_startup", Box::new(CheckBox::new(set)));
        }

        let mut setting_names = setting_names.to_vec();
        setting_names.sort();
        for name in setting_names {
            let control = match settings.get_raw(name) {
                SettingValue::Color(c) => TextBox::new(rgba_string(&c)),
                SettingValue::F32(f) => TextBox::new(f.to_string()),
                SettingValue::I32(i) => TextBox::new(i.to_string()),
                SettingValue::String(s) => TextBox::new(s.clone()),
            };
            controls.insert(name, Box::new(control));
        }

        SettingsModal { settings: controls }
    }
}

impl ModalContent for SettingsModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_height(&self, _: f32) -> f32 {
        (FONT_SIZE + MARGIN) * self.settings.len() as f32
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rectangle, piet: &mut Graphics2D) {
        let mut y = rect.top();
        for (k, v) in &self.settings {
            let rect = Rectangle::from_tuples((rect.left(), y), (rect.right(), y + FONT_SIZE));
            v.render(piet, settings, &rect, &k, self.settings.is_focused(&v));
            y += FONT_SIZE + MARGIN;
        }
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        if !self.settings.action(action) {
            if let Some(focus) = self.settings.focus() {
                focus.action(action);
                return ModalResult::None;
            }
        }
        default_modal_action(action)
    }
}

pub fn on_settings_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<SettingsModal>().unwrap();
        for (name, control) in &content.settings {
            if name == "run_at_startup_REMOVE_ME" { //TODO fix set_run_at_startup and remove
                let value = bool::from_str(control.value()).unwrap();
                crate::platform_layer::set_run_at_startup(STARTUP_TASK, value).display_failure("Unable to save settings", state);
            } else if name == "run_at_startup" {

            } else {
                state.settings.set_setting(&name, control.value()).display_failure("Unable to save settings", state);
            }
        }
        state.settings.serialize().display_failure("Unable to save settings", state);
    }
}