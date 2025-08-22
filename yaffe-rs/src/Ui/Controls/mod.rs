use crate::{LogicalPosition, Rect};
use speedy2d::color::Color;

mod checkbox;
mod image;
mod label;
mod list;
mod text_box;
pub use self::image::Image;
pub use checkbox::CheckBox;
pub use label::Label;
pub use list::{List, ListItem};
pub use text_box::TextBox;

pub const MARGIN: f32 = 10.;
pub const LABEL_SIZE: f32 = 250.;
pub const MENU_BACKGROUND: Color = Color::from_rgba(0.2, 0.2, 0.2, 0.7);
pub const MODAL_OVERLAY_COLOR: Color = Color::from_rgba(0., 0., 0., 0.6);
pub const MODAL_BACKGROUND: Color = Color::from_rgba(0.1, 0.1, 0.1, 1.);

pub fn change_brightness(color: &Color, factor: f32) -> Color {
    let r = color.r();
    let g = color.g();
    let b = color.b();
    let a = color.a();

    let (r, g, b) = if factor < 0. {
        let factor = 1. + factor;
        ((r * factor), (g * factor), (b * factor))
    } else {
        ((1. - r) * factor + r, (1. - g) * factor + g, (1. - b) * factor + b)
    };

    Color::from_rgba(r, g, b, a)
}

fn draw_label_and_box(graphics: &mut crate::Graphics, pos: &LogicalPosition, size: f32, label: &str) -> Rect {
    let font_size = graphics.font_size();
    graphics.simple_text(*pos, label);

    let min = LogicalPosition::new(pos.x + LABEL_SIZE, pos.y);
    let max = LogicalPosition::new(pos.x + LABEL_SIZE + size, pos.y + font_size);

    let control = Rect::new(min, max);
    let base = graphics.accent_color();
    let factor = graphics.dark_shade_factor();
    graphics.draw_rectangle(control, change_brightness(&base, factor));

    control
}
