use crate::{colors::*, Actions, LogicalPosition, ui::*};
use crate::widgets::get_drawable_text;
use crate::settings::SettingsFile;
use crate::modals::outline_rectangle;
use crate::utils::{Rect};

mod text_box;
mod checkbox;
mod focus_group;
mod list;
pub use text_box::TextBox;
pub use checkbox::CheckBox;
pub use focus_group::{FocusGroup, FocusGroupIter};
pub use list::{List, ListItem};

/// Provides functionality for a basic UI control (textbox, checkbox, etc...)
pub trait UiControl {
    fn render(&self, graphics: &mut crate::Graphics, settings: &SettingsFile, container: &Rect, label: &str, focused: bool);
    fn value(&self) -> &str;
    fn action(&mut self, action: &Actions);
}

fn draw_label_and_box(graphics: &mut crate::Graphics, settings: &SettingsFile, pos: &LogicalPosition, size: f32, label: &str, focused: bool) -> Rect {
    let font_size = crate::font::get_font_size(settings, graphics);
    let label = get_drawable_text(font_size, label);
    graphics.draw_text(*pos, get_font_color(settings), &label); 

    let min = LogicalPosition::new(pos.x + LABEL_SIZE, pos.y);
    let max = LogicalPosition::new(pos.x + LABEL_SIZE + size, pos.y + font_size);

    let control = Rect::new(min, max);
    let base = crate::colors::get_accent_color(settings);
    let factor = settings.get_f32(crate::SettingNames::DarkShadeFactor);
    graphics.draw_rectangle(control, change_brightness(&base, factor));
    
    if focused {
        let light_factor = settings.get_f32(crate::SettingNames::LightShadeFactor);
        outline_rectangle(graphics, &control, 1., change_brightness(&base, light_factor));
    }
    control
}
