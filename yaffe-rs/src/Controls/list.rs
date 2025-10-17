use crate::ui::{AnimationManager, DeferredAction, LayoutElement, UiElement, WidgetId};
use crate::{Actions, Graphics, LogicalPosition, LogicalSize, Rect};

pub trait ListItem: std::marker::Sync {
    fn to_display(&self) -> String;
}

impl ListItem for String {
    fn to_display(&self) -> String { self.to_string() }
}

crate::widget!(
    pub struct List<L: ListItem> {
        pub items: Vec<L> = vec!(),
        index: usize = 0,
        highlight_offset: f32 = 0.
    }
);
impl<L: ListItem> List<L> {
    pub fn from(items: Vec<L>) -> List<L> {
        let mut list = List::new();
        list.items = items;
        list
    }

    pub fn get_selected(&self) -> &L { &self.items[self.index] }

    fn move_index(&mut self, new_index: usize, animations: &mut AnimationManager) {
        let item_size = self.size.y / self.items.len() as f32;
        self.index = new_index;

        // TODO need font size because using self.size.y doesnt work if we arent shrink
        animations
            .animate(self, crate::offset_of!(List<L> => highlight_offset), item_size * self.index as f32)
            .duration(0.1)
            .start();
    }
}

impl<T: 'static, L: ListItem> UiElement<T> for List<L> {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize {
        LogicalSize::new(graphics.bounds.width(), self.items.len() as f32 * graphics.font_size())
    }

    fn action(
        &mut self,
        _state: &mut T,
        animations: &mut AnimationManager,
        action: &Actions,
        _handler: &mut DeferredAction<T>,
    ) -> bool {
        match action {
            Actions::Down => {
                if self.index < self.items.len() - 1 {
                    self.move_index(self.index + 1, animations);
                } else {
                    self.move_index(0, animations);
                }
                true
            }
            Actions::Up => {
                if self.index > 0 {
                    self.move_index(self.index - 1, animations);
                } else {
                    self.move_index(self.items.len() - 1, animations);
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

        let rect = Rect::point_and_size(
            LogicalPosition::new(pos.x, pos.y + self.highlight_offset),
            LogicalSize::new(rect.width(), font_size),
        );
        graphics.draw_rectangle(rect, graphics.accent_color());

        //Item list
        for item in self.items.iter() {
            let display = item.to_display();

            graphics.simple_text(pos, &display);
            pos.y += font_size;
        }
    }
}
