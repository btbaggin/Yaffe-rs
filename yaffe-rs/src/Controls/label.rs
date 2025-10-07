use crate::ui::{get_drawable_text, get_drawable_text_with_wrap, AnimationManager, LayoutElement, UiElement, WidgetId};
use crate::{Actions, Graphics, LogicalSize};
use speedy2d::font::FormattedTextBlock;

crate::widget!(
    pub struct Label {
        text: String = String::new(),
        wrap: bool = false,
        font_size: Option<f32> = None
    }
);

impl Label {
    pub fn simple(text: &str) -> Label {
        let mut label = Label::new();
        label.text = text.to_string();
        label
    }

    pub fn wrapping(text: &str, size: Option<f32>) -> Label {
        let mut label = Label::new();
        label.text = text.to_string();
        label.wrap = true;
        label.font_size = size;
        label
    }
}
impl<T: 'static, D: 'static> UiElement<T, D> for Label {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize {
        let text = self.get_text(graphics);
        let size = text.size();
        LogicalSize::new(size.x, size.y)
    }

    fn render(&mut self, graphics: &mut Graphics, _: &T, _: &WidgetId) {
        let rect = self.layout();
        let text = self.get_text(graphics);
        graphics.draw_text_cropped(*rect.top_left(), rect, graphics.font_color(), &text);
    }

    fn action(&mut self, _: &mut T, _: &mut AnimationManager, _: &Actions, _: &mut D) -> bool { false }
}
impl Label {
    fn get_text(&self, graphics: &mut Graphics) -> FormattedTextBlock {
        let size = self.font_size.unwrap_or(graphics.font_size());
        if self.wrap {
            get_drawable_text_with_wrap(graphics, size, &self.text, graphics.bounds.width() * graphics.scale_factor)
        } else {
            get_drawable_text(graphics, size, &self.text)
        }
    }
}