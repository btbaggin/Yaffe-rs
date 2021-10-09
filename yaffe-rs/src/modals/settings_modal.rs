use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::{font::*, ui::*, YaffeState, Actions};
use crate::modals::*;
use crate::controls::*;
use crate::logger::{UserMessage, LogTypes};
use std::collections::HashMap;
use crate::settings::{SettingsFile, SettingValue};

const STARTUP_TASK: &str = "Yaffe";

pub struct SettingsModal {
    settings: HashMap<String, Box<dyn UiControl>>,
    focus: Option<String>,
}
impl SettingsModal {
    pub fn new(settings: &SettingsFile, plugin: Option<&str>) -> SettingsModal {
        //TODO update this settings thing!
        let mut controls: HashMap<String, Box<dyn UiControl>> = HashMap::new();
        let settings = settings.get_full_settings(plugin);
        if let None = plugin {
            let set = match crate::platform_layer::get_run_at_startup(STARTUP_TASK) {
                Ok(v) => v,
                Err(e) => {
                    crate::logger::log_entry_with_message(LogTypes::Error, e, "Unable to get if Yaffe runs at startup");
                    false
                }
            };
            controls.insert(String::from("run_at_startup"), Box::new(CheckBox::new(set)));
        }

        for (k, v) in settings {
            let control = match v {
                SettingValue::Color(c) => TextBox::new(format!("{},{},{},{}", c.r(), c.g(), c.b(), c.a())),
                SettingValue::F32(f) => TextBox::new(f.to_string()),
                SettingValue::I32(i) => TextBox::new(i.to_string()),
                SettingValue::String(s) => TextBox::new(s.clone()),
            };
            controls.insert(k.to_string(), Box::new(control));
        }

        SettingsModal { settings: controls, focus: None }
    }
}

impl ModalContent for SettingsModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_height(&self) -> f32 {
        (FONT_SIZE + MARGIN) * self.settings.len() as f32
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rectangle, piet: &mut Graphics2D) {
        let mut y = rect.top();
        for (k, v) in &self.settings {
            let rect = Rectangle::from_tuples((rect.left(), y), (rect.right(), y + FONT_SIZE));
            let focused = match &self.focus {
                Some(f) => f == k,
                None => false,
            };
            v.render(piet, settings, &rect, &k, focused);
            y += FONT_SIZE + MARGIN;
        }
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        match action {
            Actions::Up => {
                self.focus = move_focus(self, false);
                ModalResult::None
            },
            Actions::Down => {
                self.focus = move_focus(self, true);
                ModalResult::None
            },
            _ => {
                if let Some(focus) = &self.focus {
                    if let Some(control) = self.settings.get_mut(focus){
                        control.action(action);
                    }
                }
                default_modal_action(action)
            } ,
        }
    }
}

fn move_focus(modal: &SettingsModal, next: bool) -> Option<String> {
    let mut keys = modal.settings.keys();
    let key = match &modal.focus {
        None => if next { keys.next() } else { keys.last() },
        Some(focus) => {
            //don't use keys reference because position advances the iterator
            let i = modal.settings.keys().position(|k| k == focus).unwrap();

            if next { keys.skip(i + 1).next() } 
            else { 
                if i == 0 { None } 
                else { keys.skip(i - 1).next() }
            }
        },
    };

    match key {
        Some(c) => Some(c.clone()),
        None => None
    }
}

pub fn on_settings_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<SettingsModal>().unwrap();
        //TODO crate::platform_layer::set_run_at_startup(STARTUP_TASK, content.run_at_startup).display_failure("Unable to set Yaffe to run at startup", state);
    }
}