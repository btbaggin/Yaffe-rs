use crate::assets::{AssetKey, Fonts, Images};
use crate::ui::MARGIN;
use crate::{Graphics, LogicalPosition, LogicalSize, Rect, ScaleFactor};
use speedy2d::color::Color;
use speedy2d::font::{FormattedTextBlock, TextAlignment, TextLayout, TextOptions};

pub struct RightAlignment {
    right: LogicalPosition,
}
impl RightAlignment {
    pub fn new(start: LogicalPosition) -> RightAlignment { RightAlignment { right: start } }
    pub fn text(mut self, graphics: &mut Graphics, text: &str) -> Self {
        let text = get_drawable_text(graphics, graphics.font_size(), text);
        let size = LogicalSize::new(text.width().to_logical(graphics), text.height().to_logical(graphics));
        self.right.x -= size.x;

        graphics.draw_text(self.right, graphics.font_color(), &text);
        self
    }

    pub fn colored_text(mut self, graphics: &mut Graphics, text: &str, color: Color) -> Self {
        let text = get_drawable_text(graphics, graphics.font_size(), text);
        let size = LogicalSize::new(text.width().to_logical(graphics), text.height().to_logical(graphics));
        self.right.x -= size.x;

        graphics.draw_text(self.right, color, &text);
        self
    }

    pub fn image(mut self, graphics: &mut Graphics, image: Images, size: LogicalSize) -> Self {
        self.right.x -= size.x;
        graphics.draw_image(Rect::point_and_size(self.right, LogicalSize::new(size.y, size.y)), image);
        self
    }

    pub fn space(mut self) -> Self {
        self.right.x -= MARGIN;
        self
    }
}

//
// Text helper methods
//

/// Simple helper method to get a text object
pub fn get_drawable_text(graphics: &mut Graphics, size: f32, text: &str) -> FormattedTextBlock {
    let font = graphics.request_font(Fonts::Regular);
    font.layout_text(text, size, TextOptions::new())
}

/// Simple helper method to get a text object that is wrapped to a certain size
pub fn get_drawable_text_with_wrap(graphics: &mut Graphics, size: f32, text: &str, width: f32) -> FormattedTextBlock {
    let font = graphics.request_font(Fonts::Regular);
    let option = TextOptions::new().with_wrap_to_width(width, TextAlignment::Left);
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
