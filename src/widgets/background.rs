use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::{YaffeState, widget};

widget!(pub struct Background {});
impl super::Widget for Background {
    fn render(&mut self, state: &YaffeState, rect: Rectangle, _: f32, piet: &mut Graphics2D) { 
        let base = crate::colors::get_accent_color(&state.settings);

        let mut queue = self.queue.borrow_mut();
        if let Some(i) = crate::assets::request_image(piet, &mut queue, crate::assets::Images::Background) {
            piet.draw_rectangle_image_tinted(rect, base, i.get_handle());
        }
    }
}