use crate::{Rect, LogicalSize, Actions};
use crate::ui::{get_accent_color, get_font_size};

pub trait ListItem: std::marker::Sync {
    fn to_display(&self) -> String;
}

impl ListItem for String {
    fn to_display(&self) -> String {
        self.to_string()
    }
}

/// Allow displaying a list of items that can be selected
/// Items must implement `ListItem` trait
pub struct List<T: ListItem> {
    pub items: Vec<T>,
    index: usize,
    first_index: std::cell::RefCell<usize>,
}
impl<T: ListItem> List<T> {
    pub fn new(items: Vec<T>) -> List<T> {
        List {
            items,
            index: 0,
            first_index: std::cell::RefCell::new(0),
        }
    }

    pub fn get_selected(&self) -> &T {
        &self.items[self.index]
    }
}

impl<T: ListItem> List<T> {
    pub fn update(&mut self, action: &Actions) -> bool {
        match action {
            Actions::Down => if self.index < self.items.len() - 1 { self.index += 1; } else { self.index = 0; }
            Actions::Up => if self.index > 0 { self.index -= 1; } else { self.index = self.items.len() - 1; }
            _ => return false,
        }

        true
    }

    pub fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        let mut pos = *rect.top_left();
        let font_size = get_font_size(settings, graphics);

        let mut first_index = self.first_index.borrow_mut();
        if self.index as f32 * font_size > rect.height() {
            *first_index += 1
        } else if self.index < *first_index {
            *first_index = self.index;
        }

        //Item list
        for (i, item) in self.items.iter().enumerate() {
            let display = item.to_display();

            if self.index == i {
                let rect = Rect::point_and_size(pos, LogicalSize::new(rect.width(), font_size));
                graphics.draw_rectangle(rect, get_accent_color(settings));
            }

            graphics.simple_text(pos, settings, &display);
            pos.y += font_size;
        }
    }
}