use crate::assets::{AssetKey, AssetTypes, Images};
use crate::ui::{AnimationManager, DeferredAction, LayoutElement, UiElement, WidgetId};
use crate::utils::Rect;
use crate::{Actions, Graphics};

crate::widget!(
    pub struct Image { image: AssetKey = AssetKey::Static(AssetTypes::Image(Images::Placeholder)) }
);

impl Image {
    pub fn from(key: AssetKey) -> Image {
        let mut image = Image::new();
        image.image = key;
        image
    }
}
impl<T: 'static> UiElement<T> for Image {
    fn render(&mut self, graphics: &mut Graphics, _: &T, _: &WidgetId) {
        let rect = self.layout();
        let image_size = crate::ui::image_fill(graphics, &self.image, &rect.size());
        graphics.draw_asset_image(Rect::point_and_size(*rect.top_left(), image_size), &self.image);
    }

    fn action(&mut self, _: &mut T, _: &mut AnimationManager, _: &Actions, _: &mut DeferredAction<T>) -> bool { false }
}
