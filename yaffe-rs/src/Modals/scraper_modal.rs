use crate::input::Actions;
use crate::scraper::{GameScrapeResult, PlatformScrapeResult};
use crate::ui::{
    AnimationManager, ContainerSize, LayoutElement, List, ListItem, ModalAction, ModalContent, UiContainer, UiElement,
    WidgetId,
};
use crate::widgets::InfoPane;
use crate::{Graphics, LogicalSize, YaffeState};

crate::widget!(
    pub struct ScraperModal<L: ListItem> {
        container: UiContainer<(), ModalAction> = UiContainer::column(),
        list_id: WidgetId = WidgetId::random(),
        info_id: WidgetId = WidgetId::random(),
        build: Option<fn(item: &L) -> InfoPane<(), ModalAction>> = None
    }
);

impl<L: ListItem> ScraperModal<L> {
    pub fn from(items: Vec<L>, builder: fn(item: &L) -> InfoPane<(), ModalAction>) -> ScraperModal<L> {
        let mut modal = ScraperModal::new();

        let list = List::from(items);
        modal.list_id = list.get_id();

        let item = list.get_selected();
        let info = builder(item);
        modal.build = Some(builder);
        modal.info_id = info.get_id();

        modal
            .container
            .with_child(UiContainer::row(), ContainerSize::Percent(0.60))
            .add_child(list, ContainerSize::Percent(0.40))
            .add_child(info, ContainerSize::Fill);
        modal
    }
}

impl<L: ListItem> UiElement<(), ModalAction> for ScraperModal<L> {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize { self.container.calc_size(graphics) }

    fn action(
        &mut self,
        state: &mut (),
        animations: &mut AnimationManager,
        action: &Actions,
        handler: &mut ModalAction,
    ) -> bool {
        if handler.close_if_accept(action) {
            return true;
        }
        if self.container.action(state, animations, action, handler) {
            let list = crate::convert_to!(self.container.find_widget(self.list_id).unwrap(), List<L>);
            let item = list.get_selected();

            let info = self.build.unwrap()(item);
            let new_info_id = info.get_id();

            self.container.replace_child(self.info_id, info);
            self.info_id = new_info_id;
            return true;
        }
        false
    }

    fn render(&mut self, graphics: &mut Graphics, state: &(), current_focus: &WidgetId) {
        self.container.render(graphics, state, current_focus);
    }
}

pub fn on_platform_found_close(state: &mut YaffeState, result: bool, content: &ModalContent) {
    if result {
        let content = content.as_any().downcast_ref::<ScraperModal<PlatformScrapeResult>>().unwrap();

        let list =
            crate::convert_to!(content.container.find_widget(content.list_id).unwrap(), List<PlatformScrapeResult>);
        let item = list.get_selected();
        crate::platform::insert_platform(state, &item.info);
    }
}

pub fn on_game_found_close(state: &mut YaffeState, result: bool, content: &ModalContent) {
    if result {
        let content = content.as_any().downcast_ref::<ScraperModal<GameScrapeResult>>().unwrap();

        let list = crate::convert_to!(content.container.find_widget(content.list_id).unwrap(), List<GameScrapeResult>);
        let item = list.get_selected();
        crate::platform::insert_game(state, &item.info, item.boxart.clone());
    }
}
