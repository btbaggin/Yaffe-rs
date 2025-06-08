use crate::{YaffeState, widget};

widget!(pub struct Background {});
impl crate::ui::Widget for Background {
    fn render(&mut self, graphics: &mut crate::Graphics, _: &YaffeState) { 
        let base = graphics.accent_color();

        graphics.draw_image_tinted(base, graphics.bounds, crate::assets::Images::Background);
    //     // if let Some(i) = crate::assets::request_image(graphics, crate::assets::Images::Background) {
    //     //     i.render_tinted(graphics, base, graphics.bounds);
    //     // }
    }
}