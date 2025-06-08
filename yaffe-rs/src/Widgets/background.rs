use crate::assets::Images;
use crate::{widget, YaffeState};

widget!(
    pub struct Background {}
);
impl crate::ui::Widget for Background {
    fn render(&mut self, graphics: &mut crate::Graphics, _: &YaffeState) {
        let base = graphics.accent_color();
        graphics.draw_image_tinted(base, graphics.bounds, Images::Background);
    }
}
