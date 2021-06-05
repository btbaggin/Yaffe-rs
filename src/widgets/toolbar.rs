use druid_shell::kurbo::{Rect, Size};
use druid_shell::piet::{Piet, TextLayout};
use crate::{YaffeState, create_widget};
use crate::colors::*;
use crate::assets::{Images};
use crate::widgets::Shifter;

create_widget!(Toolbar, );
impl super::Widget for Toolbar {
    fn layout(&self, space: &Rect, ratio_size: Size) -> Rect { 
        Rect::new(space.x0, space.y1 - space.height() * ratio_size.height, space.x1, space.y1)
    }

    fn render(&mut self, state: &YaffeState, rect: Rect, piet: &mut Piet) { 
        let time = chrono::Local::now();

        //Draw time
        let time_string = time.format("%I:%M%p");
        let text = super::get_drawable_text(piet, crate::font::FONT_SIZE, &time_string.to_string(), get_font_color(&state.settings));

        let mut right = druid_shell::kurbo::Point::new(rect.x1 - crate::ui::MARGIN, rect.y1 - text.size().height);
        right = super::right_aligned_text(piet, right, None, text).shift_x(-crate::ui::MARGIN * 2.);

        //Draw buttons
        //What actions we can perform depend on what's focused
        if state.focused_widget == crate::get_widget_id!(crate::widgets::AppList) {
            let text = super::get_drawable_text(piet, crate::font::FONT_SIZE, "Filter", get_font_color(&state.settings));
            right = super::right_aligned_text(piet, right, Some(Images::ButtonY), text).shift_x(-crate::ui::MARGIN * 2.);

            let text = super::get_drawable_text(piet, crate::font::FONT_SIZE, "Back", get_font_color(&state.settings));
            super::right_aligned_text(piet, right, Some(Images::ButtonB), text);

        } else if state.focused_widget == crate::get_widget_id!(crate::widgets::PlatformList) {
            let platform = state.get_platform();
            if crate::platform::PlatformType::Recents != platform.kind {
                let text = super::get_drawable_text(piet, crate::font::FONT_SIZE, "Info", get_font_color(&state.settings));
                right = super::right_aligned_text(piet, right, Some(Images::ButtonX), text).shift_x(-crate::ui::MARGIN * 2.);
            }
            let text = super::get_drawable_text(piet, crate::font::FONT_SIZE, "Select", get_font_color(&state.settings));
            super::right_aligned_text(piet, right, Some(Images::ButtonA), text);
        }
    }
}