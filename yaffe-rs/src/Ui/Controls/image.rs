use super::Control;
use crate::Actions;
use crate::settings::SettingsFile;
use crate::utils::Rect;
use crate::assets::AssetPathType;

pub struct Image {
    image: AssetPathType
}
impl Image {
    pub fn new(image: AssetPathType) -> Image {
        Image { image }
    }
}
impl Control for Image {
    fn render(&self, graphics: &mut crate::Graphics, _: &SettingsFile, container: &Rect) -> crate::LogicalSize {
        let slot = crate::assets::get_cached_file(&self.image);
        let slot = &mut slot.borrow_mut();
        
        let image_size = crate::ui::image_fill(graphics, slot, &container.size());
        let image = crate::assets::request_asset_image(graphics, slot);
        if let Some(i) = image {
            i.render(graphics, Rect::point_and_size(*container.top_left(), image_size));
        }

        image_size
    }

    fn action(&mut self, _: &Actions) { }

}