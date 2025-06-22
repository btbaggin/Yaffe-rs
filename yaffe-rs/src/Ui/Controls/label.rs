use super::Control;
use crate::utils::Rect;
use crate::Actions;

pub struct Label {
    text: String,
    wrap: bool,
    size: Option<f32>,
}
impl Label {
    pub fn simple(text: &str) -> Label { Label { text: text.to_string(), wrap: false, size: None } }

    pub fn new(text: &str, size: Option<f32>) -> Label { Label { text: text.to_string(), wrap: false, size } }

    pub fn wrapping(text: &str, size: Option<f32>) -> Label { Label { text: text.to_string(), wrap: true, size } }
}
impl Control for Label {
    fn render(&self, graphics: &mut crate::Graphics, container: &Rect) -> crate::LogicalSize {
        let size = self.size.unwrap_or(graphics.font_size());
        let text = if self.wrap {
            crate::ui::get_drawable_text_with_wrap(
                graphics,
                size,
                &self.text,
                (container.width() - crate::ui::MARGIN) * graphics.scale_factor,
            )
        } else {
            crate::ui::get_drawable_text(graphics, size, &self.text)
        };

        graphics.draw_text_cropped(*container.top_left(), *container, graphics.font_color(), &text);

        let size = text.size();
        crate::LogicalSize::new(size.x, size.y)
    }

    fn action(&mut self, _: &Actions) {}
}
