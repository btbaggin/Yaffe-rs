use crate::controls::TextBox;
use crate::logger::{PanicLogEntry, UserMessage};
use crate::modals::{ModalContentElement, ModalInputHandler};
use crate::ui::{ContainerSize, LayoutElement, ValueElement, WidgetId};
use crate::{DeferredAction, YaffeState};
use std::collections::HashMap;

pub struct PlatformDetailModal {
    control_map: HashMap<String, WidgetId>,
    platform_id: i64,
}

impl PlatformDetailModal {
    pub fn emulator() -> ModalContentElement<YaffeState> { PlatformDetailModal::_init(0, "", "", "") }

    pub fn from_existing(plat: &crate::TileGroup) -> ModalContentElement<YaffeState> {
        //This should never fail since we orignally got it from the database
        let platform_id = plat.id;
        let (path, args) = crate::data::PlatformInfo::get_info(platform_id).log_and_panic();

        PlatformDetailModal::_init(platform_id, &plat.name.clone(), &path, &args)
    }

    fn _init(platform_id: i64, name: &str, path: &str, args: &str) -> ModalContentElement<YaffeState> {
        let name = TextBox::from("Name", name);
        let executable = TextBox::from("Executable", path);
        let args = TextBox::from("Args", args);

        let mut control_map = HashMap::new();
        control_map.insert("Name".to_string(), name.get_id());
        control_map.insert("Executable".to_string(), executable.get_id());
        control_map.insert("Args".to_string(), args.get_id());

        let detail = PlatformDetailModal { control_map, platform_id };
        let mut modal = ModalContentElement::new(detail, true);
        modal
            .add_child(name, ContainerSize::Shrink)
            .add_child(executable, ContainerSize::Shrink)
            .add_child(args, ContainerSize::Shrink);
        modal
    }
}

impl ModalInputHandler<YaffeState> for PlatformDetailModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
}

pub fn on_add_platform_close(
    state: &mut YaffeState,
    result: bool,
    content: &ModalContentElement<YaffeState>,
    handler: &mut DeferredAction<YaffeState>,
) {
    if result {
        let details = content.get_handler::<PlatformDetailModal>();
        let name = details.control_map["Name"];
        let exe = details.control_map["Executable"];
        let args = details.control_map["Args"];
        let name = crate::convert_to!(content.find_widget(name).unwrap(), TextBox);
        let exe = crate::convert_to!(content.find_widget(exe).unwrap(), TextBox);
        let args = crate::convert_to!(content.find_widget(args).unwrap(), TextBox);
        let job = crate::Job::SearchPlatform {
            name: name.value().trim().to_string(),
            path: exe.value().trim().to_string(),
            args: args.value().trim().to_string(),
        };
        state.queue.start_job(job);

        handler.display_toast("Searching for platform information...", 2.);
    }
}

pub fn on_update_platform_close(
    state: &mut YaffeState,
    result: bool,
    content: &ModalContentElement<YaffeState>,
    handler: &mut DeferredAction<YaffeState>,
) {
    if result {
        let details = content.get_handler::<PlatformDetailModal>();
        state.refresh_list = true;

        let exe = details.control_map["Executable"];
        let args = details.control_map["Args"];
        let exe = crate::convert_to!(content.find_widget(exe).unwrap(), TextBox);
        let args = crate::convert_to!(content.find_widget(args).unwrap(), TextBox);
        crate::data::PlatformInfo::update(details.platform_id, exe.value().trim(), args.value().trim())
            .display_failure("Unable to update platform", handler);
    }
}
