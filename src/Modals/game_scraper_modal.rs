
use crate::{YaffeState, Rect, LogicalSize, LogicalPosition};
use crate::scraper::GameScrapeResult;
use crate::input::Actions;
use crate::assets::AssetKey;
use crate::ui::{List, Label, Image, Container, Control, ModalResult, ModalContent, ModalSize};


pub struct GameScraperModal {
    list: List<GameScrapeResult>,
    details: Container
}
impl GameScraperModal {
    pub fn new(items: Vec<GameScrapeResult>) -> GameScraperModal {
        GameScraperModal {
            details: build_container(&items[0]),
            list: List::new(items),
        }
    }
}

impl ModalContent for GameScraperModal {
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

fn build_container(item: &GameScrapeResult) -> Container {
    let mut main = Container::vertical(0.5);
    main.add(Label::new(item.info.name.clone(), Some(crate::ui::TITLE_SIZE)));

    let mut top = Container::horizontal(0.25);
    let mut details = Container::vertical(1.);

    top.add(Image::new(AssetKey::Url(item.boxart.clone())));
    details.add(Label::simple(format!("Players: {}", item.info.players)));
    details.add(Label::simple(format!("Rating: {}", item.info.rating)));
    details.add(Label::simple(format!("Released: {}", item.info.released)));
    top.add(details);
    main.add(top);
    main.add(Label::wrapping(item.info.overview.clone(), None));
    
    main
}

pub fn on_game_found_close(state: &mut YaffeState, result: ModalResult, content: &dyn ModalContent, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<GameScraperModal>().unwrap();

        let item = content.list.get_selected();
        crate::platform::insert_game(state, &item.info, item.boxart.clone());
    }
}