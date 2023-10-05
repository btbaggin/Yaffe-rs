use crate::{YaffeState, widget, LogicalPosition};
use crate::assets::Images;
use crate::ui::MARGIN;

widget!(pub struct Toolbar {});
impl crate::ui::Widget for Toolbar {
    fn render(&mut self, graphics: &mut crate::Graphics, state: &YaffeState) { 
        let time = chrono::Local::now();
        let rect = graphics.bounds;

        let font_size = graphics.font_size();
        let font_color = graphics.font_color();

        //Draw time
        let time_string = time.format("%I:%M%p");
        let text = crate::ui::get_drawable_text(font_size, &time_string.to_string());

        let mut right = LogicalPosition::new(rect.right() - MARGIN, rect.bottom() - text.height());
        right = crate::ui::right_aligned_text(graphics, right, None, font_color, text);
        right = LogicalPosition::new(right.x - MARGIN * 2., right.y);

        //Draw buttons
        //What actions we can perform depend on what's focused
        if state.focused_widget == crate::get_widget_id!(crate::widgets::AppList) {
            let text = crate::ui::get_drawable_text(font_size, "Filter");
            right = crate::ui::right_aligned_text(graphics, right, Some(Images::ButtonY), font_color, text);
            right = LogicalPosition::new(right.x - MARGIN * 2., right.y);

            let text = crate::ui::get_drawable_text(font_size, "Back");
            crate::ui::right_aligned_text(graphics, right, Some(Images::ButtonB), font_color, text);

        } else if state.focused_widget == crate::get_widget_id!(crate::widgets::PlatformList) {
            let platform = state.get_platform();
            if crate::platform::PlatformType::Recents != platform.kind {
                let text = crate::ui::get_drawable_text(font_size, "Settings");
                right = crate::ui::right_aligned_text(graphics, right, Some(Images::ButtonX), font_color, text);
                right = LogicalPosition::new(right.x - MARGIN * 2., right.y);
            }
            let text = crate::ui::get_drawable_text(font_size, "Select");
            crate::ui::right_aligned_text(graphics, right, Some(Images::ButtonA), font_color, text);
        }
    }
}