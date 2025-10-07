use crate::{LogicalPosition, Rect};
use speedy2d::color::Color;

mod checkbox;
mod image;
mod label;
mod list;
mod passbox;
mod text_box;
pub use checkbox::CheckBox;
pub use image::Image;
pub use label::Label;
pub use list::{List, ListItem};
pub use passbox::{PassBox, RestrictedPasscode};
pub use text_box::TextBox;

pub const LABEL_SIZE: f32 = 250.;
pub const MENU_BACKGROUND: Color = Color::from_rgba(0.2, 0.2, 0.2, 0.7);
pub const MODAL_OVERLAY_COLOR: Color = Color::from_rgba(0., 0., 0., 0.6);
pub const MODAL_BACKGROUND: Color = Color::from_rgba(0.1, 0.1, 0.1, 1.);

fn draw_label_and_box(graphics: &mut crate::Graphics, pos: &LogicalPosition, size: f32, label: &str) -> Rect {
    let font_size = graphics.font_size();
    graphics.simple_text(*pos, label);

    let min = LogicalPosition::new(pos.x + LABEL_SIZE, pos.y);
    let max = LogicalPosition::new(pos.x + LABEL_SIZE + size, pos.y + font_size);

    let control = Rect::new(min, max);
    let base = graphics.accent_color();
    let factor = graphics.dark_shade_factor();
    graphics.draw_rectangle(control, crate::ui::change_brightness(&base, factor));

    control
}
