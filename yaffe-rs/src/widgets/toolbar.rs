use crate::{YaffeState, widget, LogicalPosition};
use crate::assets::Images;
use crate::ui::{MARGIN, get_font_color, get_font_size};

widget!(pub struct Toolbar {});
impl crate::ui::Widget for Toolbar {
    fn render(&mut self, graphics: &mut crate::Graphics, state: &YaffeState) { 
        let time = chrono::Local::now();
        let rect = graphics.bounds;

        let font_size = get_font_size(&state.settings, graphics);

        //Draw time
        let time_string = time.format("%I:%M%p");
        let text = crate::ui::get_drawable_text(font_size, &time_string.to_string());

        let mut right = LogicalPosition::new(rect.right() - MARGIN, rect.bottom() - text.height());
        right = crate::ui::right_aligned_text(graphics, right, None, get_font_color(&state.settings), text);
        right = LogicalPosition::new(right.x - MARGIN * 2., right.y);

        //Draw buttons
        //What actions we can perform depend on what's focused
        if crate::is_widget_focused!(state, crate::widgets::AppList) {
            let text = crate::ui::get_drawable_text(font_size, "Filter");
            right = crate::ui::right_aligned_text(graphics, right, Some(Images::ButtonY), get_font_color(&state.settings), text);
            right = LogicalPosition::new(right.x - MARGIN * 2., right.y);

            let text = crate::ui::get_drawable_text(font_size, "Back");
            crate::ui::right_aligned_text(graphics, right, Some(Images::ButtonB), get_font_color(&state.settings), text);

        } else if crate::is_widget_focused!(state, crate::widgets::PlatformList) {
            let platform = state.get_platform();
            if crate::platform::PlatformType::Recents != platform.kind {
                let text = crate::ui::get_drawable_text(font_size, "Settings");
                right = crate::ui::right_aligned_text(graphics, right, Some(Images::ButtonX), get_font_color(&state.settings), text);
                right = LogicalPosition::new(right.x - MARGIN * 2., right.y);
            }
            let text = crate::ui::get_drawable_text(font_size, "Select");
            crate::ui::right_aligned_text(graphics, right, Some(Images::ButtonA), get_font_color(&state.settings), text);
        }
    }
}