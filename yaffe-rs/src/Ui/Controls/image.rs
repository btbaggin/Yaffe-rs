use super::Control;
use crate::assets::AssetKey;
use crate::utils::Rect;
use crate::Actions;

pub struct Image {
    image: AssetKey,
}
impl Image {
    pub fn new(image: AssetKey) -> Image { Image { image } }
}
impl Control for Image {
    fn render(&self, graphics: &mut crate::Graphics, container: &Rect) -> crate::LogicalSize {
        let image_size = crate::ui::image_fill(graphics, &self.image, &container.size());
        graphics.draw_asset_image(Rect::point_and_size(*container.top_left(), image_size), &self.image);

        image_size
    }

    fn action(&mut self, _: &Actions) {}
}
