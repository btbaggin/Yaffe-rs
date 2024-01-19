use crate::{YaffeState, Actions, Rect};
use crate::modals::*;
use crate::logger::{PanicLogEntry, UserMessage};
use crate::ui::{Control, TextBox, Container};

pub struct PlatformDetailModal {
    controls: Container,
    id: i64,
}
impl PlatformDetailModal {
    pub fn emulator() -> PlatformDetailModal {
        let mut controls = Container::vertical(1.);
        controls.add_field("Name", TextBox::from_str("Name".to_string(), ""));
        controls.add_field("Executable", TextBox::from_str("Executable".to_string(), ""));
        controls.add_field("Args", TextBox::from_str("Args".to_string(), ""));

        PlatformDetailModal { controls, id: 0, }
    }

    pub fn from_existing(plat: &crate::Platform) -> PlatformDetailModal {
        //This should never fail since we orignally got it from the database
        let id = plat.id.unwrap();
        let (path, args) = crate::data::PlatformInfo::get_info(id).log_and_panic();

        let mut controls = Container::vertical(1.);
        controls.add_field("Name", TextBox::new("Name".to_string(), plat.name.clone()));
        controls.add_field("Executable", TextBox::new("Executable".to_string(), path));
        controls.add_field("Args", TextBox::new("Args".to_string(), args));

        PlatformDetailModal { controls, id, }
    }
}

impl ModalContent for PlatformDetailModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, rect: Rect, graphics: &crate::Graphics) -> LogicalSize {
        let height = (graphics.font_size() + MARGIN) * self.controls.child_count() as f32;
        LogicalSize::new(Self::modal_width(rect, ModalSize::Half), height)
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        self.controls.action(action);
        Self::default_modal_action(action)
    }

    fn render(&self, rect: Rect, graphics: &mut crate::Graphics) {
        self.controls.render(graphics, &rect);
    }
}

pub fn on_add_platform_close(state: &mut YaffeState, result: ModalResult, content: &dyn ModalContent, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<PlatformDetailModal>().unwrap();

        let job_id = crate::job_system::generate_job_id();

        let lock = state.queue.lock().log_and_panic();
        let mut queue = lock.borrow_mut();

        let name = content.controls.by_tag("Name").unwrap();
        let exe = content.controls.by_tag("Executable").unwrap();
        let args = content.controls.by_tag("Args").unwrap();
        let job = crate::Job::SearchPlatform {
            id: job_id,
            name: name.value().to_string(),
            path: exe.value().to_string(),
            args: args.value().to_string()
        };
        queue.send(job).unwrap();

        state.toasts.insert(job_id, String::from("Searching for platform information..."));
    }
}

pub fn on_update_platform_close(state: &mut YaffeState, result: ModalResult, content: &dyn ModalContent, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<PlatformDetailModal>().unwrap();
        state.refresh_list = true;

        let exe = content.controls.by_tag("Executable").unwrap();
        let args = content.controls.by_tag("Args").unwrap();
		crate::data::PlatformInfo::update(content.id, exe.value(), args.value())
            .display_failure("Unable to update platform", state);
    }
}
