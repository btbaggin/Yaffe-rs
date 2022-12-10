use crate::{YaffeState, Rect, LogicalSize};
use crate::data::PlatformInfo;
use crate::input::Actions;
use crate::settings::SettingsFile;
use crate::modals::ListModal;
use crate::ui::{List, ModalResult, ModalContent, ModalSize};

pub struct PlatformScraperModal {
    list: List<PlatformInfo>,
}
impl PlatformScraperModal {
    pub fn new(items: Vec<PlatformInfo>) -> PlatformScraperModal {
        PlatformScraperModal { list: List::new(items) }
    }
}

impl ModalContent for PlatformScraperModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, _: &SettingsFile, rect: Rect, _: &crate::Graphics) -> LogicalSize {
        LogicalSize::new(Self::modal_width(rect, ModalSize::Full), rect.height())
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        self.list.update(action);
        Self::default_modal_action(action)
    }

    fn render(&self, settings: &SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        //TODO add more stuff
        self.list.render(settings, rect, graphics);
    }
}

pub fn on_platform_found_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<ListModal<crate::data::PlatformInfo>>().unwrap();

        let item = content.get_selected();
        crate::platform::insert_platform(state, item);
    }
}
