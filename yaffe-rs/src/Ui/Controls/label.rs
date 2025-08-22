use crate::ui::{AnimationManager, LayoutElement, UiElement, WidgetId};
use crate::{Actions, Graphics, LogicalSize};

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
        let size = self.font_size.unwrap_or(graphics.font_size());
        let width = graphics.bounds.width();
        // TODO im double drawing. eewww
        let text = if self.wrap {
            crate::ui::get_drawable_text_with_wrap(
                graphics,
                size,
                &self.text,
                (width - crate::ui::MARGIN) * graphics.scale_factor,
            )
        } else {
            crate::ui::get_drawable_text(graphics, size, &self.text)
        };

        let size = text.size();
        LogicalSize::new(size.x, size.y)
    }

    fn render(&mut self, graphics: &mut Graphics, _: &T, _: &WidgetId) {
        let rect = self.layout();
        let size = self.font_size.unwrap_or(graphics.font_size());
        let text = if self.wrap {
            crate::ui::get_drawable_text_with_wrap(
                graphics,
                size,
                &self.text,
                (rect.width() - crate::ui::MARGIN) * graphics.scale_factor,
            )
        } else {
            crate::ui::get_drawable_text(graphics, size, &self.text)
        };

        graphics.draw_text_cropped(*rect.top_left(), rect, graphics.font_color(), &text);
    }

    fn action(&mut self, _: &mut T, _: &mut AnimationManager, _: &Actions, _: &mut D) -> bool { false }
}
