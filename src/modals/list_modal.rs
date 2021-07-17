use crate::{Rect, V2, Actions};
use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::colors::*;
use crate::modals::{ModalResult, ModalContent, default_modal_action};

pub struct ListModal<T: ListItem> {
    items: Vec<T>,
    title: Option<String>,
    index: usize,
}
impl<T: ListItem> ListModal<T> {
    pub fn new(title: Option<String>) -> ListModal<T> {
        ListModal {
            items: Vec::new(),
            title: title,
            index: 0,
        }
    }

    pub fn add_item(&mut self, item: T) {
        self.items.push(item);
    }
}

pub trait ListItem: std::marker::Sync {
    fn to_display(&self) -> String;
}

impl ListItem for String {
    fn to_display(&self) -> String {
        self.to_string()
    }
}

impl<T: 'static + ListItem> ModalContent for ListModal<T> {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_height(&self) -> f32 {
        let mut count = self.items.len();
        if let Some(_) = self.title {
            count += 1;
        }
        count as f32 * 30.
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        match action {
            Actions::Down => {
                if self.index < self.items.len() - 1 { self.index += 1; }
                ModalResult::None
            }
            Actions::Up => {
                if self.index > 0 { self.index -= 1; }
                ModalResult::None
            }
            _ => default_modal_action(action)
        }
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rectangle, piet: &mut Graphics2D) {
        let mut pos = *rect.top_left();

        //Title
        if let Some(t) = &self.title {
            let title_label = crate::widgets::get_drawable_text(crate::font::FONT_SIZE, &t);
            piet.draw_text(pos, get_font_color(settings), &title_label);
            pos.y += 30.;
        }

        //Item list
        for (i, item) in self.items.iter().enumerate() {
            let display = item.to_display();

            if self.index == i {
                let rect = Rect::point_and_size(pos, V2::new(rect.width(), 30.));
                piet.draw_rectangle(rect, get_accent_color(settings));
            }

            let item_label = crate::widgets::get_drawable_text(crate::font::FONT_SIZE, &display);
            piet.draw_text(pos, get_font_color(settings), &item_label);
            pos.y += 30.;
        }
    }
}

impl<T: ListItem> ListModal<T> {
    pub fn get_selected(&self) -> &T {
        &self.items[self.index]
    }
}