use crate::assets::AssetKey;
use crate::input::Actions;
use crate::scraper::PlatformScrapeResult;
use crate::ui::{Container, Control, Image, Label, List, ModalContent, ModalResult, ModalSize};
use crate::{LogicalPosition, LogicalSize, Rect, YaffeState};

pub struct PlatformScraperModal {
    list: List<PlatformScrapeResult>,
    details: Container,
}
impl PlatformScraperModal {
    pub fn new(items: Vec<PlatformScrapeResult>) -> PlatformScraperModal {
        PlatformScraperModal { details: build_container(&items[0]), list: List::new(items) }
    }
}

impl ModalContent for PlatformScraperModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, rect: Rect, _: &crate::Graphics) -> LogicalSize {
        LogicalSize::new(Self::modal_width(rect, ModalSize::Full), rect.height())
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        if self.list.update(action) {
            let item = self.list.get_selected();
            self.details = build_container(item);
        }
        Self::default_modal_action(action)
    }

    fn render(&self, rect: Rect, graphics: &mut crate::Graphics) {
        let half_size = LogicalSize::new(rect.width() / 2., rect.height());

        let size = self.details.render(graphics, &rect);
        let right = Rect::point_and_size(LogicalPosition::new(rect.left() + size.x, rect.top()), half_size);
        self.list.render(right, graphics);
    }
}

fn build_container(item: &PlatformScrapeResult) -> Container {
    let mut main = Container::vertical(0.5);
    let mut top = Container::horizontal(0.25);
    top.add(Image::new(AssetKey::Url(item.boxart.clone())));

    main.add(Label::new(&item.info.platform.clone(), Some(crate::ui::TITLE_SIZE)));
    main.add(top);
    main.add(Label::wrapping(&item.overview.clone(), None));

    main
}

pub fn on_platform_found_close(
    state: &mut YaffeState,
    result: ModalResult,
    content: &dyn ModalContent,
    _: &mut crate::DeferredAction,
) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<PlatformScraperModal>().unwrap();

        let item = content.list.get_selected();
        crate::platform::insert_platform(state, &item.info);
    }
}
