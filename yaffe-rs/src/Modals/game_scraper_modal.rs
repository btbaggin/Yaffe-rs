use crate::assets::AssetKey;
use crate::input::Actions;
use crate::scraper::GameScrapeResult;
use crate::ui::{ContainerSize, UiContainer, Image, Label, List, UiElement, LayoutElement, WidgetId, AnimationManager, ModalAction, ModalContent};
use crate::{LogicalPosition, LogicalSize, Rect, YaffeState, Graphics};

crate::widget!(
    pub struct GameScraperModal {
        container: UiContainer<(), ModalAction> = UiContainer::row()
    }
);

impl GameScraperModal {
    pub fn from(items: Vec<GameScrapeResult>) -> GameScraperModal {
        let mut modal = GameScraperModal::new();

        let item = &items[0].clone();
        modal.container
            .add_child(List::from(items), ContainerSize::Fill)
            .with_child(UiContainer::column(), ContainerSize::Fill)
                .add_child(Image::from(AssetKey::Url(item.boxart.clone())), ContainerSize::Percent(0.25))
                .with_child(UiContainer::column(), ContainerSize::Fill)
                    .add_child(Label::simple(&format!("Players: {}", item.info.players)), ContainerSize::Shrink)
                    .add_child(Label::simple(&format!("Rating: {}", item.info.rating)), ContainerSize::Shrink)
                    .add_child(Label::simple(&format!("Released: {}", item.info.released)), ContainerSize::Shrink)
                    .add_child(Label::wrapping(&item.info.overview.clone(), None), ContainerSize::Shrink);
        // let mut main = Container::vertical(0.5);
        // main.add(Label::new(&item.info.name.clone(), Some(crate::ui::TITLE_SIZE)));

        // let mut top = Container::horizontal(0.25);
        // let mut details = Container::vertical(1.);

        // top.add(Image::new(AssetKey::Url(item.boxart.clone())));
        // details.add(Label::simple(&format!("Players: {}", item.info.players)));
        // details.add(Label::simple(&format!("Rating: {}", item.info.rating)));
        // details.add(Label::simple(&format!("Released: {}", item.info.released)));
        // top.add(details);
        // main.add(top);
        // main.add(Label::wrapping(&item.info.overview.clone(), None));

        // main

        // modal.details = build_container(&items[0]);
        modal
    }
}

impl UiElement<(), ModalAction> for GameScraperModal {
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
        // Self::default_modal_action(action)
        handler.close_if_accept(action)
    }

    fn render(&mut self, graphics: &mut Graphics, state: &(), current_focus: &WidgetId) {
        self.container.render(graphics, state, current_focus);
        // let rect = self.layout();
        // let half_size = LogicalSize::new(rect.width() / 2., rect.height());

        // let size = self.details.render(graphics, &rect);
        // let right = Rect::point_and_size(LogicalPosition::new(rect.left() + size.x, rect.top()), half_size);
        // self.list.render(right, graphics);
    }
}

pub fn on_game_found_close(state: &mut YaffeState, result: bool, content: &ModalContent) {
    if result {
        let content = content.as_any().downcast_ref::<GameScraperModal>().unwrap();

        // let item = content.list.get_selected();
        // crate::platform::insert_game(state, &item.info, item.boxart.clone());
    }
}
