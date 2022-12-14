use super::{Control, get_font_size, get_font_color};
use crate::Actions;
use crate::settings::SettingsFile;
use crate::utils::Rect;

pub struct Label {
    text: String,
    wrap: bool,
    size: Option<f32>
}
impl Label {
    pub fn simple(text: String) -> Label {
        Label { text, wrap: false, size: None }
    }

    pub fn new(text: String, size: Option<f32>) -> Label {
        Label { text, wrap: false, size }
    }

    pub fn wrapping(text: String, size: Option<f32>) -> Label {
        Label { text, wrap: true, size }
    }
}
impl Control for Label {
    fn render(&self, graphics: &mut crate::Graphics, settings: &SettingsFile, container: &Rect) -> crate::LogicalSize {
        let size = if let Some(size) = self.size { size } else { get_font_size(settings, graphics) };
        let text = if self.wrap {
            crate::ui::get_drawable_text_with_wrap(size, &self.text, (container.width() - crate::ui::MARGIN) * graphics.scale_factor)
        } else {
            crate::ui::get_drawable_text(size, &self.text)
        };

        graphics.draw_text(*container.top_left(), get_font_color(settings), &text);
        
        let size = text.size();
        crate::LogicalSize::new(size.x, size.y)
    }

    fn action(&mut self, _: &Actions) { }

}