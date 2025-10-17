use crate::controls::{List, ListItem};
use crate::input::Actions;
use crate::modals::{ModalContentElement, ModalInputHandler};
use crate::scraper::{GameScrapeResult, PlatformScrapeResult};
use crate::ui::{AnimationManager, ContainerSize, LayoutElement, UiContainer, UiElement, WidgetId};
use crate::widgets::InfoPane;
use crate::{DeferredAction, YaffeState};

pub struct ScraperModal<L: ListItem> {
    list_id: WidgetId,
    info_id: WidgetId,
    builder: fn(item: &L) -> InfoPane<YaffeState>,
    platform: bool,
}

impl<L: ListItem + 'static> ScraperModal<L> {
    pub fn from(
        items: Vec<L>,
        platform: bool,
        builder: fn(item: &L) -> InfoPane<YaffeState>,
    ) -> ModalContentElement<YaffeState> {
        let list = List::from(items);
        let item = list.get_selected();
        let info = builder(item);

        let content = ScraperModal { list_id: list.get_id(), info_id: info.get_id(), builder, platform };
        let mut modal = ModalContentElement::new(content, false);
        modal
            .with_child(UiContainer::row(), ContainerSize::Percent(0.60))
            .add_child(list, ContainerSize::Percent(0.40))
            .add_child(info, ContainerSize::Fill);
        modal
    }
}

impl<L: ListItem + 'static> ModalInputHandler<YaffeState> for ScraperModal<L> {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn action(
        &mut self,
        state: &mut YaffeState,
        animations: &mut AnimationManager,
        action: &Actions,
        handler: &mut DeferredAction<YaffeState>,
        container: &mut UiContainer<YaffeState>,
    ) -> bool {
        if container.action(state, animations, action, handler) {
            let list = crate::convert_to!(container.find_widget(self.list_id).unwrap(), List<L>);
            let item = list.get_selected();

            let info = (self.builder)(item);
            let new_info_id = info.get_id();

            container.replace_child(self.info_id, info);
            self.info_id = new_info_id;
            return true;
        }
        false
    }

    fn on_close(
        &self,
        state: &mut YaffeState,
        result: bool,
        content: &UiContainer<YaffeState>,
        _: &mut DeferredAction<YaffeState>,
    ) {
        if !result {
            return;
        }
        if self.platform {
            let list = crate::convert_to!(content.find_widget(self.list_id).unwrap(), List<PlatformScrapeResult>);
            let item = list.get_selected();
            crate::platform::insert_platform(state, &item.info);
        } else {
            let list = crate::convert_to!(content.find_widget(self.list_id).unwrap(), List<GameScrapeResult>);
            let item = list.get_selected();
            crate::platform::insert_game(state, &item.info, item.boxart.clone());
        }
    }
}
