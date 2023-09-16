use crate::{Rect, LogicalPosition};
use crate::settings::{SettingsFile, SettingNames};
use crate::job_system::ThreadSafeJobQueue;
use crate::ui::get_drawable_text;
use speedy2d::color::Color;
use yaffe_lib::SettingValue;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub struct Graphics<'a> {
    graphics: &'a mut speedy2d::Graphics2D,
    pub queue: ThreadSafeJobQueue,
    pub scale_factor: f32,
    pub bounds: Rect,
    pub delta_time: f32,
    cached_settings: HashMap<SettingNames, SettingValue>
}
impl<'a> Graphics<'a> {
    pub fn new(graphics: &'a mut speedy2d::Graphics2D, queue: ThreadSafeJobQueue, scale_factor: f32, bounds: Rect, delta_time: f32) -> Graphics {
        Graphics { graphics, queue, scale_factor, bounds, delta_time, cached_settings: HashMap::new() }
    }
    pub fn cache_settings(&mut self, settings: &SettingsFile) {
        for s in [SettingNames::InfoFontSize, SettingNames::LightShadeFactor, SettingNames::DarkShadeFactor] {
            self.cached_settings.insert(s, SettingValue::F32(settings.get_f32(s)));
        }
        for s in [SettingNames::FontColor, SettingNames::AccentColor] {
            self.cached_settings.insert(s, SettingValue::Color(settings.get_color(s)));
        }
    }

    pub fn dark_shade_factor(&self) -> f32 { if let SettingValue::F32(c) = self.cached_settings[&SettingNames::DarkShadeFactor] { c } else  { unreachable!() } }
    pub fn light_shade_factor(&self) -> f32 { if let SettingValue::F32(c) = self.cached_settings[&SettingNames::LightShadeFactor] { c } else  { unreachable!() } }
    pub fn accent_color(&self) -> Color { if let SettingValue::Color(c) = self.cached_settings[&SettingNames::AccentColor] { c } else  { unreachable!() } }
    pub fn font_color(&self) -> Color { if let SettingValue::Color(c) = self.cached_settings[&SettingNames::FontColor] { c } else  { unreachable!() } }
    pub fn font_size(&self) -> f32 {
        let size = if let SettingValue::F32(s) = self.cached_settings[&SettingNames::InfoFontSize] { s } else  { unreachable!() };
        size * self.scale_factor
    }

    pub fn draw_rectangle(&mut self, rect: Rect, color: Color) {
        self.graphics.draw_rectangle(rect.to_physical(self.scale_factor), color);
    }
    pub fn draw_text(&mut self, position: LogicalPosition, color: Color, text: &Rc<speedy2d::font::FormattedTextBlock>) {
        self.graphics.draw_text(position.to_physical(self.scale_factor), color, text);
    }
    pub fn simple_text(&mut self, position: LogicalPosition, text: &str) {
        let text = &get_drawable_text(self.font_size(), text);
        self.graphics.draw_text(position.to_physical(self.scale_factor), self.font_color(), text);
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