use super::Control;
use crate::Actions;
use crate::utils::Rect;
use crate::assets::AssetKey;

pub struct Image {
    image: AssetKey
}
impl Image {
    pub fn new(image: AssetKey) -> Image {
        Image { image }
    }
}
impl Control for Image {
    fn render(&self, graphics: &mut crate::Graphics, container: &Rect) -> crate::LogicalSize {  
        let image_size = crate::ui::image_fill(graphics, &self.image, &container.size());
        let image = crate::assets::request_asset_image(graphics, &self.image);
        if let Some(i) = image {
            i.render(graphics, Rect::point_and_size(*container.top_left(), image_size));
        }

        image_size
    }

    fn action(&mut self, _: &Actions) { }

}