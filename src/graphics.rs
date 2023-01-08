use crate::{Rect, LogicalPosition};
use crate::settings::SettingsFile;
use crate::job_system::ThreadSafeJobQueue;
use crate::ui::{get_font_color, get_font_size, get_drawable_text};
use speedy2d::color::Color;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub struct Graphics<'a> {
    pub graphics: &'a mut speedy2d::Graphics2D,
    pub queue: Option<ThreadSafeJobQueue>,
    pub scale_factor: f32,
    pub bounds: Rect,
    pub delta_time: f32,
}
impl<'a> Graphics<'a> {
    pub fn draw_rectangle(&mut self, rect: Rect, color: Color) {
        self.graphics.draw_rectangle(rect.to_physical(self.scale_factor), color);
    }
    pub fn draw_text(&mut self, position: LogicalPosition, color: Color, text: &Rc<speedy2d::font::FormattedTextBlock>) {
        self.graphics.draw_text(position.to_physical(self.scale_factor), color, text);
    }
    pub fn simple_text(&mut self, position: LogicalPosition, settings: &SettingsFile, text: &str) {
        let text = &get_drawable_text(get_font_size(settings, self), text);
        self.graphics.draw_text(position.to_physical(self.scale_factor), get_font_color(settings), text);
    }
    pub fn draw_line(&mut self, pos1: LogicalPosition, pos2: LogicalPosition, width: f32, color: Color) {
        self.graphics.draw_line(pos1.to_physical(self.scale_factor), pos2.to_physical(self.scale_factor), width, color);
    }
    pub fn draw_text_cropped(&mut self, position: LogicalPosition, rect: Rect, color: Color, text: &Rc<speedy2d::font::FormattedTextBlock>) {
        self.graphics.draw_text_cropped(position.to_physical(self.scale_factor), rect.to_physical(self.scale_factor), color, text);

    }
}
impl<'a> Deref for Graphics<'a> {
    type Target = speedy2d::Graphics2D;
    fn deref(&self) -> &Self::Target {
        self.graphics
    }
}
impl<'a> DerefMut for Graphics<'a> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        self.graphics
    }
}