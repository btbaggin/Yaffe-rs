use crate::{YaffeState, widget};
use crate::assets::Images;

widget!(pub struct Background {});
impl crate::ui::Widget for Background {
    fn render(&mut self, graphics: &mut crate::Graphics, _: &YaffeState) { 
        let base = graphics.accent_color();
        graphics.draw_image_tinted(base, graphics.bounds, Images::Background);
    }
}