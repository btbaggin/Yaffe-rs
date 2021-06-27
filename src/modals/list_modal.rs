use druid_shell::kurbo::{Rect};
use druid_shell::piet::{Piet, RenderContext};
use crate::Actions;
use crate::colors::*;
use crate::modals::{ModalResult, ModalContent, default_modal_action, DeferredModalAction};

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
    fn get_height(&self) -> f64 {
        let mut count = self.items.len();
        if let Some(_) = self.title {
            count += 1;
        }
        count as f64 * 30.
    }

    fn action(&mut self, action: &Actions, _: &mut DeferredModalAction) -> ModalResult {
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

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, piet: &mut Piet) {
        let mut pos = druid_shell::kurbo::Point::new(rect.x0, rect.y0);

        //Title
        if let Some(t) = &self.title {
            let title_label = crate::widgets::get_drawable_text(piet, crate::font::FONT_SIZE, &t, get_font_color(settings));
            piet.draw_text(&title_label, pos);
            pos.y += 30.;
        }

        //Item list
        for (i, item) in self.items.iter().enumerate() {
            let display = item.to_display();

            if self.index == i {
                let rect = Rect::new(pos.x, pos.y, rect.x1, pos.y + 30.);
                piet.fill(rect, &get_accent_color(settings));
            }

            let item_label = crate::widgets::get_drawable_text(piet, crate::font::FONT_SIZE, &display, get_font_color(settings));
            piet.draw_text(&item_label, pos);
            pos.y += 30.;
        }
    }
}

impl<T: ListItem> ListModal<T> {
    pub fn get_selected(&self) -> &T {
        &self.items[self.index]
    }
}