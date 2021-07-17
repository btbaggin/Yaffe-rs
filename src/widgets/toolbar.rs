use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::{YaffeState, create_widget, V2};
use crate::colors::*;
use crate::Rect;
use crate::assets::{Images};
use crate::widgets::Shifter;

create_widget!(Toolbar, );
impl super::Widget for Toolbar {
    fn layout(&self, space: &Rectangle, ratio_size: V2) -> Rectangle { 
        Rectangle::from_tuples((space.left(), space.bottom() - space.height() * ratio_size.y), (space.right(), space.bottom()))
    }

    fn render(&mut self, state: &YaffeState, rect: Rectangle, _: f32, piet: &mut Graphics2D) { 
        let time = chrono::Local::now();

        //Draw time
        let time_string = time.format("%I:%M%p");
        let text = super::get_drawable_text(crate::font::FONT_SIZE, &time_string.to_string());

        let mut right = V2::new(rect.right() - crate::ui::MARGIN, rect.bottom() - text.height());
        right = super::right_aligned_text(piet, right, None, get_font_color(&state.settings), text).shift_x(-crate::ui::MARGIN * 2.);

        //Draw buttons
        //What actions we can perform depend on what's focused
        if state.focused_widget == crate::get_widget_id!(crate::widgets::AppList) {
            let text = super::get_drawable_text(crate::font::FONT_SIZE, "Filter");
            right = super::right_aligned_text(piet, right, Some(Images::ButtonY), get_font_color(&state.settings), text).shift_x(-crate::ui::MARGIN * 2.);

            let text = super::get_drawable_text(crate::font::FONT_SIZE, "Back");
            super::right_aligned_text(piet, right, Some(Images::ButtonB), get_font_color(&state.settings), text);

        } else if state.focused_widget == crate::get_widget_id!(crate::widgets::PlatformList) {
            let platform = state.get_platform();
            if crate::platform::PlatformType::Recents != platform.kind {
                let text = super::get_drawable_text(crate::font::FONT_SIZE, "Info");
                right = super::right_aligned_text(piet, right, Some(Images::ButtonX), get_font_color(&state.settings), text).shift_x(-crate::ui::MARGIN * 2.);
            }
            let text = super::get_drawable_text(crate::font::FONT_SIZE, "Select");
            super::right_aligned_text(piet, right, Some(Images::ButtonA), get_font_color(&state.settings), text);
        }
    }
}