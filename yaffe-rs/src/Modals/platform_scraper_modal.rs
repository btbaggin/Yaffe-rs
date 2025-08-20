use crate::assets::AssetKey;
use crate::input::Actions;
use crate::scraper::PlatformScrapeResult;
use crate::ui::{Image, Label, List, UiElement, WidgetId, AnimationManager, LayoutElement, ModalAction, ModalContent, UiContainer};
use crate::{LogicalPosition, LogicalSize, Rect, YaffeState, Graphics};

crate::widget!(
    pub struct PlatformScraperModal {
        list: List<PlatformScrapeResult> = List::from(vec!()),
        details: UiContainer<(), ModalAction> = UiContainer::column()
    }
);

// TODO
// impl PlatformScraperModal {
//     pub fn new(items: Vec<PlatformScrapeResult>) -> PlatformScraperModal {
//         PlatformScraperModal { details: build_container(&items[0]), list: List::new(items) }
//     }
// }

impl UiElement<(), ModalAction> for PlatformScraperModal {
    // fn as_any(&self) -> &dyn std::any::Any { self }
    // fn size(&self, rect: Rect, _: &crate::Graphics) -> LogicalSize {
    //     LogicalSize::new(Self::modal_width(rect, ModalSize::Full), rect.height())
    // }

    fn action(&mut self, _state: &mut (), _: &mut AnimationManager, action: &Actions, handler: &mut ModalAction) -> bool {
        // TODO
        // if self.list.update(action) {
        //     let item = self.list.get_selected();
        //     self.details = build_container(item);
        // }
        handler.close_if_accept(action)
    }

    fn render(&mut self, graphics: &mut Graphics, _: &(), _: &WidgetId) {
        // let rect = self.layout();
        // let half_size = LogicalSize::new(rect.width() / 2., rect.height());

        // let size = self.details.render(graphics, &rect);
        // let right = Rect::point_and_size(LogicalPosition::new(rect.left() + size.x, rect.top()), half_size);
        // self.list.render(right, graphics);
    }
}

// fn build_container(item: &PlatformScrapeResult) -> Container {
//     let mut main = Container::vertical(0.5);
//     let mut top = Container::horizontal(0.25);
//     top.add(Image::new(AssetKey::Url(item.boxart.clone())));

//     main.add(Label::new(&item.info.platform.clone(), Some(crate::ui::TITLE_SIZE)));
//     main.add(top);
//     main.add(Label::wrapping(&item.overview.clone(), None));

//     main
// }

pub fn on_platform_found_close(
    state: &mut YaffeState,
    result: bool,
    content: &ModalContent,
) {
    if result {
        let content = content.as_any().downcast_ref::<PlatformScraperModal>().unwrap();

        let item = content.list.get_selected();
        crate::platform::insert_platform(state, &item.info);
    }
}
