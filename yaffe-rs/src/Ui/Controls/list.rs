use crate::{Actions, LogicalSize, Rect, Graphics};
use crate::ui::{WidgetId, AnimationManager, UiElement, LayoutElement};

pub trait ListItem: std::marker::Sync {
    fn to_display(&self) -> String;
}

impl ListItem for String {
    fn to_display(&self) -> String { self.to_string() }
}

crate::widget!(
    pub struct List<L: ListItem> {
        pub items: Vec<L> = vec!(),
        index: usize = 0
    }
);
impl<L: ListItem> List<L> {
    pub fn from(items: Vec<L>) -> List<L> {
        let mut list = List::new();
        list.items = items;
        list
    }

    pub fn get_selected(&self) -> &L { &self.items[self.index] }
}

impl<T: 'static, D: 'static, L: ListItem> UiElement<T, D> for List<L> {
    fn action(&mut self, _state: &mut T, _: &mut AnimationManager, action: &Actions, _handler: &mut D) -> bool {
        match action {
            Actions::Down => {
                if self.index < self.items.len() - 1 {
                    self.index += 1;
                } else {
                    self.index = 0;
                }
                true
            }
            Actions::Up => {
                if self.index > 0 {
                    self.index -= 1;
                } else {
                    self.index = self.items.len() - 1;
                }
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, graphics: &mut Graphics, _: &T, _: &WidgetId) {
        let rect = self.layout();
        let mut pos = *rect.top_left();
        let font_size = graphics.font_size();
        let height = font_size * self.items.len() as f32;

        //Item list
        for (i, item) in self.items.iter().enumerate() {
            let display = item.to_display();

            if self.index == i {
                let rect = Rect::point_and_size(pos, LogicalSize::new(rect.width(), font_size));
                graphics.draw_rectangle(rect, graphics.accent_color());
            }

            graphics.simple_text(pos, &display);
            pos.y += font_size;
        }

        self.set_layout(Rect::point_and_size(*rect.top_left(), LogicalSize::new(rect.width(), height)));
    }
}
