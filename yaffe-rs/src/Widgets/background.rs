use crate::assets::Images;
use crate::ui::{WidgetId, Widget};
use crate::{widget, YaffeState, DeferredAction};

widget!(
    pub struct Background {}
);
impl Widget<YaffeState, DeferredAction> for Background {
    fn render(&mut self, graphics: &mut crate::Graphics, _: &YaffeState, _: &WidgetId) {
        let base = graphics.accent_color();
        graphics.draw_image_tinted(base, graphics.bounds, Images::Background);
    }
}
