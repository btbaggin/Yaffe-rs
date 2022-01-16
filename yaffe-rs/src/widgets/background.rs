use crate::{YaffeState, widget};
use crate::logger::PanicLogEntry;

widget!(pub struct Background {});
impl super::Widget for Background {
    fn render(&mut self, graphics: &mut crate::Graphics, state: &YaffeState) { 
        let base = crate::colors::get_accent_color(&state.settings);

        let lock = self.queue.lock().log_and_panic();
        let mut queue = lock.borrow_mut();
        if let Some(i) = crate::assets::request_image(graphics, &mut queue, crate::assets::Images::Background) {
            let rect = graphics.bounds.to_physical(graphics.scale_factor);
            graphics.graphics.draw_rectangle_image_tinted(rect, base, i.get_handle());
        }
    }
}