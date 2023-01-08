use crate::{Actions, LogicalPosition};
use crate::settings::SettingsFile;
use crate::utils::Rect;
use speedy2d::color::Color;

mod text_box;
mod checkbox;
mod list;
mod container;
mod image;
mod label;
pub use text_box::TextBox;
pub use checkbox::CheckBox;
pub use list::{List, ListItem};
pub use container::Container;
pub use self::image::Image;
pub use label::Label;

pub const MARGIN: f32 = 10.;
pub const TITLE_SIZE: f32 = 36.;
pub const LABEL_SIZE: f32 = 250.;
pub const MENU_BACKGROUND: Color = Color::from_rgba(0.2, 0.2, 0.2, 0.7);
pub const MODAL_OVERLAY_COLOR: Color = Color::from_rgba(0., 0., 0., 0.6);
pub const MODAL_BACKGROUND: Color = Color::from_rgba(0.1, 0.1, 0.1, 1.);

/// Provides functionality for a basic UI control (textbox, checkbox, etc...)
pub trait Control {
    fn render(&self, graphics: &mut crate::Graphics, settings: &SettingsFile, container: &Rect) -> crate::LogicalSize;
    fn action(&mut self, action: &Actions);
}

pub trait InputControl: Control {
    fn value(&self) -> &str;
    fn set_focused(&mut self, value: bool);
}

pub fn get_font_size(settings: &crate::settings::SettingsFile, graphics: &crate::Graphics) -> f32 {
    settings.get_f32(crate::SettingNames::InfoFontSize) * graphics.scale_factor
}

pub fn get_font_color(settings: &crate::settings::SettingsFile) -> Color {
    settings.get_color(crate::SettingNames::FontColor)
}
pub fn get_font_unfocused_color(settings: &crate::settings::SettingsFile) -> Color {
    let color = settings.get_color(crate::SettingNames::FontColor);
    change_brightness(&color, -0.4)
}

pub fn get_accent_color(settings: &crate::settings::SettingsFile) -> Color {
    settings.get_color(crate::SettingNames::AccentColor)
}
pub fn get_accent_unfocused_color(settings: &crate::settings::SettingsFile) -> Color {
    let color = settings.get_color(crate::SettingNames::AccentColor);
    change_brightness(&color, -0.3)
}

pub fn change_brightness(color: &Color, factor: f32) -> Color {
    let mut r = color.r();
    let mut g = color.g();
    let mut b = color.b();
    let a = color.a();

    if factor < 0. {
        let factor = 1. + factor;
        r *= factor;
        g *= factor;
        b *= factor;
    } else {
        r = (1. - r) * factor + r;
        g = (1. - g) * factor + g;
        b  = (1. - b) * factor + b;
    }

    Color::from_rgba(r, g, b, a)
}

pub fn rgba_string(c: &Color) -> String {
    format!("{},{},{},{}", c.r(), c.g(), c.b(), c.a())
}

fn draw_label_and_box(graphics: &mut crate::Graphics, settings: &SettingsFile, pos: &LogicalPosition, size: f32, label: &str) -> Rect {
    let font_size = get_font_size(settings, graphics);
    graphics.simple_text(*pos, settings, label); 

    let min = LogicalPosition::new(pos.x + LABEL_SIZE, pos.y);
    let max = LogicalPosition::new(pos.x + LABEL_SIZE + size, pos.y + font_size);

    let control = Rect::new(min, max);
    let base = get_accent_color(settings);
    let factor = settings.get_f32(crate::SettingNames::DarkShadeFactor);
    graphics.draw_rectangle(control, change_brightness(&base, factor));
    
    control
}
