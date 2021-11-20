use speedy2d::Graphics2D;
use crate::{YaffeState, widget};
use crate::widgets::RenderState;

widget!(pub struct Background {});
impl super::Widget for Background {
    fn render(&mut self, graphics: &mut Graphics2D, state: &YaffeState, render_state: RenderState) { 
        let base = crate::colors::get_accent_color(&state.settings);

        let mut queue = self.queue.borrow_mut();
        if let Some(i) = crate::assets::request_image(graphics, &mut queue, crate::assets::Images::Background) {
            graphics.draw_rectangle_image_tinted(render_state.bounds.into(), base, i.get_handle());
        }
    }
}