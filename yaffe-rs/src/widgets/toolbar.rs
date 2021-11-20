use crate::{YaffeState, widget, LogicalPosition};
use crate::colors::*;
use crate::assets::{Images};
use crate::widgets::Shifter;

widget!(pub struct Toolbar {});
impl super::Widget for Toolbar {
    fn render(&mut self, graphics: &mut crate::Graphics, state: &YaffeState) { 
        let time = chrono::Local::now();
        let rect = graphics.bounds;

        //Draw time
        let time_string = time.format("%I:%M%p");
        let text = super::get_drawable_text(crate::font::FONT_SIZE, &time_string.to_string());

        let mut right = LogicalPosition::new(rect.right() - crate::ui::MARGIN, rect.bottom() - text.height());
        right = super::right_aligned_text(graphics, right, None, get_font_color(&state.settings), text).shift_x(-crate::ui::MARGIN * 2.);

        //Draw buttons
        //What actions we can perform depend on what's focused
        if state.focused_widget == crate::get_widget_id!(crate::widgets::AppList) {
            let text = super::get_drawable_text(crate::font::FONT_SIZE, "Filter");
            right = super::right_aligned_text(graphics, right, Some(Images::ButtonY), get_font_color(&state.settings), text).shift_x(-crate::ui::MARGIN * 2.);

            let text = super::get_drawable_text(crate::font::FONT_SIZE, "Back");
            super::right_aligned_text(graphics, right, Some(Images::ButtonB), get_font_color(&state.settings), text);

        } else if state.focused_widget == crate::get_widget_id!(crate::widgets::PlatformList) {
            let platform = state.get_platform();
            if crate::platform::PlatformType::Recents != platform.kind {
                let text = super::get_drawable_text(crate::font::FONT_SIZE, "Settings");
                right = super::right_aligned_text(graphics, right, Some(Images::ButtonX), get_font_color(&state.settings), text).shift_x(-crate::ui::MARGIN * 2.);
            }
            let text = super::get_drawable_text(crate::font::FONT_SIZE, "Select");
            super::right_aligned_text(graphics, right, Some(Images::ButtonA), get_font_color(&state.settings), text);
        }
    }
}