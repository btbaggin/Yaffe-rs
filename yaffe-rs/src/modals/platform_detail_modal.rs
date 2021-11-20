use speedy2d::Graphics2D;
use crate::{YaffeState, Actions, Rect};
use crate::modals::*;
use crate::logger::{PanicLogEntry, UserMessage};
use crate::controls::*;
use crate::{font::FONT_SIZE, ui::MARGIN};

pub struct PlatformDetailModal {
    controls: FocusGroup<dyn UiControl>,
    id: i64,
}
impl PlatformDetailModal {
    pub fn application() -> PlatformDetailModal {
        let mut controls: FocusGroup<dyn UiControl> = FocusGroup::new();
        controls.insert("Name", Box::new(TextBox::from_str("")));
        controls.insert("Executable", Box::new(TextBox::from_str("")));
        controls.insert("Args", Box::new(TextBox::from_str("")));
        controls.insert("Image", Box::new(TextBox::from_str("")));

        PlatformDetailModal { controls, id: 0, }
    }

    pub fn emulator() -> PlatformDetailModal {
        let mut controls: FocusGroup<dyn UiControl> = FocusGroup::new();
        controls.insert("Name", Box::new(TextBox::from_str("")));
        controls.insert("Executable", Box::new(TextBox::from_str("")));
        controls.insert("Args", Box::new(TextBox::from_str("")));
        controls.insert("Folder", Box::new(TextBox::from_str("")));

        PlatformDetailModal { controls, id: 0, }
    }

    pub fn from_existing(plat: &crate::Platform, id: i64) -> PlatformDetailModal {
        //This should never fail since we orignally got it from the database
        let (path, args, roms) = crate::database::get_platform_info(id).log_and_panic();

        let mut controls: FocusGroup<dyn UiControl> = FocusGroup::new();
        controls.insert("Name", Box::new(TextBox::new(plat.name.clone())));
        controls.insert("Executable", Box::new(TextBox::new(path)));
        controls.insert("Args", Box::new(TextBox::new(args)));
        controls.insert("Folder", Box::new(TextBox::new(roms)));

        PlatformDetailModal { controls, id, }
    }
}

impl ModalContent for PlatformDetailModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_height(&self, _: f32) -> f32 {
        (crate::font::FONT_SIZE + crate::ui::MARGIN) * 4.
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

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, piet: &mut Graphics2D) {
        let mut y = rect.top();

        for (k, v) in &self.controls {
            let rect = Rect::from_tuples((rect.left(), y), (rect.right(), y + FONT_SIZE));
            v.render(piet, settings, &rect, &k, self.controls.is_focused(&v));
            y += FONT_SIZE + MARGIN;
        }
    }
}

pub fn on_add_platform_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<PlatformDetailModal>().unwrap();

        let state_ptr = crate::RawDataPointer::new(state);
        let mut queue = state.queue.borrow_mut();

        let name = content.controls.by_tag("Name").unwrap();
        let exe = content.controls.by_tag("Executable").unwrap();
        let args = content.controls.by_tag("Args").unwrap();
        let folder = content.controls.by_tag("Folder").unwrap();
        queue.send(crate::JobType::SearchPlatform((state_ptr, name.value().to_string(), exe.value().to_string(), args.value().to_string(), folder.value().to_string())));  
    }
}

pub fn on_update_platform_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<PlatformDetailModal>().unwrap();
        state.refresh_list = true;

        let exe = content.controls.by_tag("Executable").unwrap();
        let args = content.controls.by_tag("Args").unwrap();
        let folder = content.controls.by_tag("Folder").unwrap();
		crate::database::update_platform(content.id, &exe.value(), &args.value(), &folder.value())
            .display_failure("Unable to update platform", state);
    }
}

pub fn on_platform_found_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<ListModal<crate::database::PlatformData>>().unwrap();

        let item = content.get_selected();
        crate::platform::insert_platform(state, item);
    }
}

pub fn on_game_found_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<ListModal<crate::database::GameData>>().unwrap();

        let item = content.get_selected();
        crate::platform::insert_game(state, item);
    }
}