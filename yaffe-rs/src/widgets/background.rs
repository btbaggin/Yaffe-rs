use crate::{YaffeState, widget};

widget!(pub struct Background {});
impl crate::ui::Widget for Background {
    fn render(&mut self, graphics: &mut crate::Graphics, state: &YaffeState) { 
        let base = crate::ui::get_accent_color(&state.settings);

        if let Some(i) = crate::assets::request_image(graphics, crate::assets::Images::Background) {
            i.render_tinted(graphics, base, graphics.bounds);
        }
    }
}