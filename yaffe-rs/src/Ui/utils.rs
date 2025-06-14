use speedy2d::color::Color;
use speedy2d::font::{FormattedTextBlock, TextLayout, TextOptions, TextAlignment};
use crate::{LogicalPosition, LogicalSize, ScaleFactor, Rect};
use crate::assets::{Images, Fonts, AssetKey};
use crate::graphics::Graphics;

//
// Text helper methods
//
/// Draws text that is right aligned to parameter `right`
/// If an image is passed it will be drawn to the left of the text
/// Returns the new right-most position
pub fn right_aligned_text(graphics: &mut crate::Graphics, right: LogicalPosition, image: Option<crate::assets::Images>, color: Color, text: std::rc::Rc<FormattedTextBlock>) -> LogicalPosition {
    let size = LogicalSize::new(text.width().to_logical(graphics), text.height().to_logical(graphics));
    let mut right = LogicalPosition::new(right.x - size.x, right.y);

    graphics.draw_text(right, color, &text);
    if let Some(i) = image {
        right.x -= size.y;
        graphics.draw_image(Rect::point_and_size(right, LogicalSize::new(size.y, size.y)), i);
        // if let Some(i) = crate::assets::request_image(graphics, i) {
        //     i.render(graphics, Rect::point_and_size(right, LogicalSize::new(size.y, size.y)));
        // }
    }

    right
}

/// Simple helper method to get a text object
pub fn get_drawable_text(graphics: &mut Graphics, size: f32, text: &str) -> std::rc::Rc<FormattedTextBlock> {
    let font = graphics.request_font(Fonts::Regular);
    font.layout_text(text, size, TextOptions::new())
}

/// Simple helper method to get a text object that is wrapped to a certain size
pub fn get_drawable_text_with_wrap(graphics: &mut Graphics,size: f32, text: &str, width: f32) -> std::rc::Rc<FormattedTextBlock> {
    let font = graphics.request_font(Fonts::Regular);
    let option = TextOptions::new();
    let option = option.with_wrap_to_width(width, TextAlignment::Left);
    font.layout_text(text, size, option)
}

/// Scales an image to the largest size that can fit in the smallest dimension
pub fn image_fill(graphics: &mut crate::Graphics, slot: &AssetKey, size: &LogicalSize) -> LogicalSize {
    let bitmap_size = if let Some(i) = graphics.request_asset_image(slot) {
            i.size()
    } else {
        graphics.request_image(Images::Placeholder).unwrap().size()
    };

    let bitmap_size = bitmap_size.to_logical(graphics.scale_factor);

    let mut width = bitmap_size.x;
    let mut height = bitmap_size.y;
    // first check if we need to scale width
    if bitmap_size.x > size.x {
        //scale width to fit
        width = size.x;
        //scale height to maintain aspect ratio
        height = (width * bitmap_size.y) / bitmap_size.x;
    }

    // then check if we need to scale even with the new height
    if height > size.y {
        //scale height to fit instead
        height = size.y;
        //scale width to maintain aspect ratio
        width = (height * bitmap_size.x) / bitmap_size.y;
    }

    LogicalSize::new(width, height)
}

#[macro_export]
macro_rules! is_focused {
    ($state:ident) => {
        $state.focused_widget == $crate::get_widget_id!(Self)
    }
}

pub fn outline_rectangle(graphics: &mut crate::Graphics, rect: &Rect, size: f32, color: speedy2d::color::Color) {
    let top_left = *rect.top_left();
    let bottom_right = *rect.bottom_right();
    let top_right = LogicalPosition::new(bottom_right.x, top_left.y);
    let bottom_left = LogicalPosition::new(top_left.x, bottom_right.y);

    graphics.draw_line(top_left, top_right, size, color);
    graphics.draw_line(top_right, bottom_right, size, color);
    graphics.draw_line(bottom_right, bottom_left, size, color);
    graphics.draw_line(bottom_left, top_left, size, color);
}
