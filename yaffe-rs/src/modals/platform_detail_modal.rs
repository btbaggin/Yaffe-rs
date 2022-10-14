use crate::{YaffeState, Actions, Rect};
use crate::modals::*;
use crate::logger::{PanicLogEntry, UserMessage};
use crate::ui_control::*;
use crate::ui::MARGIN;

pub struct PlatformDetailModal {
    controls: FocusGroup<dyn UiControl>,
    id: i64,
}
impl PlatformDetailModal {
    pub fn emulator() -> PlatformDetailModal {
        let mut controls: FocusGroup<dyn UiControl> = FocusGroup::new();
        controls.insert("Name", Box::new(TextBox::from_str("")));
        controls.insert("Executable", Box::new(TextBox::from_str("")));
        controls.insert("Args", Box::new(TextBox::from_str("")));

        PlatformDetailModal { controls, id: 0, }
    }

    pub fn from_existing(plat: &crate::Platform) -> PlatformDetailModal {
        //This should never fail since we orignally got it from the database
        let id = plat.id.unwrap();
        let (path, args) = crate::data::PlatformInfo::get_info(id).log_and_panic();

        let mut controls: FocusGroup<dyn UiControl> = FocusGroup::new();
        controls.insert("Name", Box::new(TextBox::new(plat.name.clone())));
        controls.insert("Executable", Box::new(TextBox::new(path)));
        controls.insert("Args", Box::new(TextBox::new(args)));

        PlatformDetailModal { controls, id, }
    }
}

impl ModalContent for PlatformDetailModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &crate::Graphics) -> LogicalSize {
        let height = (crate::font::get_font_size(settings, graphics) + crate::ui::MARGIN) * self.controls.len() as f32;
        LogicalSize::new(modal_width(rect, ModalSize::Half), height)
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        if !self.controls.action(action) {
            if let Some(focus) = self.controls.focus() {
                focus.action(action);
                return ModalResult::None;
            }
        }
        default_modal_action(action)
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        let mut y = rect.top();

        let font_size = crate::font::get_font_size(settings, graphics);
        for (k, v) in &self.controls {
            let rect = Rect::from_tuples((rect.left(), y), (rect.right(), y + font_size));
            v.render(graphics, settings, &rect, &k, self.controls.is_focused(&v));
            y += font_size + MARGIN;
        }
    }
}

pub fn on_add_platform_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<PlatformDetailModal>().unwrap();

        let state_ptr = crate::RawDataPointer::new(state);
        let lock = state.queue.lock().log_and_panic();
        let mut queue = lock.borrow_mut();

        let name = content.controls.by_tag("Name").unwrap();
        let exe = content.controls.by_tag("Executable").unwrap();
        let args = content.controls.by_tag("Args").unwrap();
        queue.send(crate::JobType::SearchPlatform((state_ptr, name.value().to_string(), exe.value().to_string(), args.value().to_string()))).unwrap();
    }
}

pub fn on_update_platform_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<PlatformDetailModal>().unwrap();
        state.refresh_list = true;

        let exe = content.controls.by_tag("Executable").unwrap();
        let args = content.controls.by_tag("Args").unwrap();
		crate::data::PlatformInfo::update(content.id, &exe.value(), &args.value())
            .display_failure("Unable to update platform", state);
    }
}
