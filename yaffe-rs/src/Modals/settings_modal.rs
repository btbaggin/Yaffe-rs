use crate::logger::{LogEntry, UserMessage};
use crate::modals::*;
use crate::settings::SettingsFile;
use crate::ui::{CheckBox, ContainerSize, ModalContent, TextBox, UiContainer, ValueElement};
use crate::{Actions, YaffeState};

const STARTUP_TASK: &str = "Yaffe";

crate::widget!(
    pub struct SettingsModal {
        names: Vec<String> = vec!(),
        settings: UiContainer<(), ModalAction> = UiContainer::column(),
        focus: Option<WidgetId> = None
    }
);

impl SettingsModal {
    pub fn from(settings: &SettingsFile) -> SettingsModal {
        let mut modal = SettingsModal::new();
        let set = crate::os::get_run_at_startup(STARTUP_TASK).log("Unable to get if Yaffe runs at startup");
        modal.names.push(String::from("run_at_startup"));
        modal.settings.add_child(CheckBox::from("run_at_startup".to_string(), set), ContainerSize::Shrink);

        let mut setting_names = settings.get_full_settings();

        setting_names.sort_by(|x, y| x.0.cmp(&y.0));
        for (name, default) in setting_names {
            modal.names.push(name.clone());
            let element = TextBox::from(&name.clone(), &default.to_string());
            modal.settings.add_child(element, ContainerSize::Shrink);
        }
        modal
    }
}

impl UiElement<(), ModalAction> for SettingsModal {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize { self.settings.calc_size(graphics) }

    fn render(&mut self, graphics: &mut Graphics, state: &(), _: &WidgetId) {
        self.settings.render(graphics, state, &self.focus.unwrap_or(WidgetId::random()));
        self.set_layout(self.settings.layout())
    }

    fn action(
        &mut self,
        state: &mut (),
        animations: &mut AnimationManager,
        action: &Actions,
        handler: &mut ModalAction,
    ) -> bool {
        let handled = match action {
            Actions::Up => {
                self.focus = self.settings.move_focus(self.focus, false);
                true
            }
            Actions::Down => {
                self.focus = self.settings.move_focus(self.focus, true);
                true
            }
            _ => false,
        };
        let close = handler.close_if_accept(action);
        if !handled && !close {
            if let Some(focus) = self.focus {
                if let Some(widget) = self.settings.find_widget_mut(focus) {
                    return widget.action(state, animations, action, handler);
                }
            }
        }
        handled || close
    }
}

pub fn on_settings_close(state: &mut YaffeState, result: bool, content: &ModalContent) {
    if result {
        let content = content.as_any().downcast_ref::<SettingsModal>().unwrap();

        for (i, name) in content.names.iter().enumerate() {
            match name.as_str() {
                "run_at_startup" => {
                    let run_at_startup = crate::convert_to!(content.settings.get_child(i), CheckBox);
                    crate::os::set_run_at_startup(STARTUP_TASK, run_at_startup.value())
                        .display_failure("Unable to save settings", state);
                }
                _ => {
                    let control = crate::convert_to!(content.settings.get_child(i), TextBox);
                    state
                        .settings
                        .set_setting(name, &control.value())
                        .display_failure("Unable to save settings", state);
                }
            };
        }

        //Save settings
        state.settings.serialize().display_failure("Unable to save settings", state);
    }
}
