use crate::logger::{PanicLogEntry, UserMessage};
use crate::modals::*;
use crate::ui::{ContainerSize, ModalContent, TextBox, UiContainer, UiElement, ValueElement};
use crate::{Actions, YaffeState};
use std::collections::HashMap;

crate::widget!(
    pub struct PlatformDetailModal {
        controls: UiContainer<(), ModalAction> = UiContainer::column(),
        control_map: HashMap<String, WidgetId> = HashMap::new(),
        platform_id: i64 = 0,
        focus: Option<WidgetId> = None
    }
);

impl PlatformDetailModal {
    pub fn emulator() -> PlatformDetailModal {
        let mut detail = PlatformDetailModal::new();
        PlatformDetailModal::_init("", "", "", &mut detail);
        detail
    }

    pub fn from_existing(plat: &crate::TileGroup) -> PlatformDetailModal {
        //This should never fail since we orignally got it from the database
        let platform_id = plat.id;
        let (path, args) = crate::data::PlatformInfo::get_info(platform_id).log_and_panic();

        let mut detail = PlatformDetailModal::new();
        PlatformDetailModal::_init(&plat.name.clone(), &path, &args, &mut detail);
        detail
    }

    fn _init(name: &str, path: &str, args: &str, modal: &mut PlatformDetailModal) {
        let name = TextBox::from("Name", name);
        let executable = TextBox::from("Executable", path);
        let args = TextBox::from("Args", args);

        modal.control_map.insert("Name".to_string(), name.get_id());
        modal.control_map.insert("Executable".to_string(), executable.get_id());
        modal.control_map.insert("Args".to_string(), args.get_id());

        modal
            .controls
            .add_child(name, ContainerSize::Shrink)
            .add_child(executable, ContainerSize::Shrink)
            .add_child(args, ContainerSize::Shrink);
    }
}

impl UiElement<(), ModalAction> for PlatformDetailModal {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize { self.controls.calc_size(graphics) }

    fn action(
        &mut self,
        state: &mut (),
        animations: &mut AnimationManager,
        action: &Actions,
        handler: &mut ModalAction,
    ) -> bool {
        // TODO this is duplicated from settings_modal. eh?
        let handled = match action {
            Actions::Up => {
                self.focus = self.controls.move_focus(self.focus, false);
                true
            }
            Actions::Down => {
                self.focus = self.controls.move_focus(self.focus, true);
                true
            }
            _ => false,
        };
        let close = handler.close_if_accept(action);
        if !handled && !close {
            if let Some(focus) = self.focus {
                if let Some(widget) = self.controls.find_widget_mut(focus) {
                    return widget.action(state, animations, action, handler);
                }
            }
        }
        handled || close
    }

    fn render(&mut self, graphics: &mut Graphics, state: &(), _: &WidgetId) {
        self.controls.render(graphics, state, &self.focus.unwrap_or(WidgetId::random()));
    }
}

pub fn on_add_platform_close(state: &mut YaffeState, result: bool, content: &ModalContent) {
    if result {
        let content = content.as_any().downcast_ref::<PlatformDetailModal>().unwrap();

        let job_id = crate::job_system::generate_job_id();

        let name = content.control_map["Name"];
        let exe = content.control_map["Executable"];
        let args = content.control_map["Args"];
        let name = crate::convert_to!(content.controls.find_widget(name).unwrap(), TextBox);
        let exe = crate::convert_to!(content.controls.find_widget(exe).unwrap(), TextBox);
        let args = crate::convert_to!(content.controls.find_widget(args).unwrap(), TextBox);
        let job = crate::Job::SearchPlatform {
            id: job_id,
            name: name.value().trim().to_string(),
            path: exe.value().trim().to_string(),
            args: args.value().trim().to_string(),
        };
        state.queue.start_job(job);

        state.display_toast(job_id, "Searching for platform information...");
    }
}

pub fn on_update_platform_close(state: &mut YaffeState, result: bool, content: &ModalContent) {
    if result {
        let content = content.as_any().downcast_ref::<PlatformDetailModal>().unwrap();
        state.refresh_list = true;

        let exe = content.control_map["Executable"];
        let args = content.control_map["Args"];
        let exe = crate::convert_to!(content.controls.find_widget(exe).unwrap(), TextBox);
        let args = crate::convert_to!(content.controls.find_widget(args).unwrap(), TextBox);
        crate::data::PlatformInfo::update(content.platform_id, exe.value().trim(), args.value().trim())
            .display_failure("Unable to update platform", state);
    }
}
