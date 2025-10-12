use crate::controls::{List, ListItem};
use crate::input::Actions;
use crate::modals::{ModalAction, ModalInputHandler, ModalContentElement};
use crate::scraper::{GameScrapeResult, PlatformScrapeResult};
use crate::ui::{AnimationManager, ContainerSize, LayoutElement, UiContainer, UiElement, WidgetId};
use crate::widgets::InfoPane;
use crate::{YaffeState, DeferredAction};

pub struct ScraperModal<L: ListItem> {
    list_id: WidgetId,
    info_id: WidgetId,
    build: fn(item: &L) -> InfoPane<(), ModalAction>,
}

impl<L: ListItem + 'static> ScraperModal<L> {
    pub fn from(items: Vec<L>, builder: fn(item: &L) -> InfoPane<(), ModalAction>) -> ModalContentElement {
        let list = List::from(items);
        let item = list.get_selected();
        let info = builder(item);

        let content = ScraperModal { list_id: list.get_id(), info_id: info.get_id(), build: builder };
        let mut modal = ModalContentElement::new(content, false);
        modal
            .with_child(UiContainer::row(), ContainerSize::Percent(0.60))
            .add_child(list, ContainerSize::Percent(0.40))
            .add_child(info, ContainerSize::Fill);
        modal
    }
}

impl<L: ListItem + 'static> ModalInputHandler for ScraperModal<L> {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn action(
        &mut self,
        animations: &mut AnimationManager,
        action: &Actions,
        handler: &mut ModalAction,
        container: &mut UiContainer<(), ModalAction>,
    ) -> bool {
        if container.action(&mut (), animations, action, handler) {
            let list = crate::convert_to!(container.find_widget(self.list_id).unwrap(), List<L>);
            let item = list.get_selected();

            let info = (self.build)(item);
            let new_info_id = info.get_id();

            container.replace_child(self.info_id, info);
            self.info_id = new_info_id;
            return true;
        }
        false
    }
}

pub fn on_platform_found_close(state: &mut YaffeState, result: bool, content: &ModalContentElement, _: &mut DeferredAction) {
    if result {
        let details = content.get_handler::<ScraperModal<PlatformScrapeResult>>();
        let list = crate::convert_to!(content.find_widget(details.list_id).unwrap(), List<PlatformScrapeResult>);
        let item = list.get_selected();
        crate::platform::insert_platform(state, &item.info);
    }
}

pub fn on_game_found_close(state: &mut YaffeState, result: bool, content: &ModalContentElement, _: &mut DeferredAction) {
    if result {
        let details = content.get_handler::<ScraperModal<GameScrapeResult>>();
        let list = crate::convert_to!(content.find_widget(details.list_id).unwrap(), List<GameScrapeResult>);
        let item = list.get_selected();
        crate::platform::insert_game(state, &item.info, item.boxart.clone());
    }
}
