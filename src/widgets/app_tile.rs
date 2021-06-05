use druid_shell::kurbo::{Rect, Size, Point};
use druid_shell::piet::{RenderContext, Piet, TextLayout};
use crate::YaffeState;
use crate::colors::*;
use crate::widgets::Shifter;

const ANIMATION_TIME: f64 = 0.25;
const SELECTED_SCALAR: f64 = 0.2;
const ROM_OUTLINE_SIZE: f64 = 7.5;

const VISIBLE_FLAG: u8 = 0x01;

pub struct AppTile { 
    queue: std::sync::Arc<crate::JobQueue>,
    index: usize,
    focused: bool,
    time: f64,
    flags: u8,
    pub position: Point,
    pub size: Size,
}
impl AppTile {
    pub fn new(q: std::sync::Arc<crate::JobQueue>, index: usize) -> AppTile {
        AppTile { 
            queue: q,
            index: index,
            focused: false,
            time: 0.,
            flags: VISIBLE_FLAG,
            position: Point::new(0., 0.),
            size: Size::new(0., 0.),
        }
    }
}
impl AppTile {
    pub fn is_visible(&self) -> bool {
        (self.flags & VISIBLE_FLAG) != 0
    }

    pub fn apply_filter(&mut self, filter: &crate::widgets::SearchInfo, apps: &Vec<crate::Executable>) {
        fn set_visible(flags: u8, visible: bool) -> u8 {
            if visible { flags | VISIBLE_FLAG } else { flags & !VISIBLE_FLAG }
        }
        let exe = &apps[self.index];
        self.flags = set_visible(self.flags, filter.item_is_visible(exe))
    }


    pub fn render(&mut self, settings: &crate::settings::SettingsFile, focused: bool, exe: &crate::Executable, piet: &mut Piet) {
        if !self.is_visible() { return; }

        let mut target_size = self.size;
        let mut position = self.position;

        if self.focused && focused {
            let animation_remainder = (ANIMATION_TIME - self.time) / ANIMATION_TIME;
            //Have alpha fade in as the time grows to full size
            let alpha = f64::powf(animation_remainder, 2.);

            target_size = target_size * (1. + animation_remainder * SELECTED_SCALAR);
            let x = position.x - (target_size.width - self.size.width) / 2.;
            let y = position.y - (target_size.height - self.size.height) / 2.;
            position = Point::new(x, y);

            //Position of the text and buttons for the focused game
            let mut menu_position = Point::new(position.x + target_size.width, position.y + target_size.height + 2.);

            let name = super::get_drawable_text_with_wrap(piet, crate::font::FONT_SIZE, &exe.name, get_font_color(settings).with_alpha(alpha), target_size.width);
            let mut height = name.size().height;

			//Check if we need to push the buttons below the text due to overlap
			if name.size().width > target_size.width * 0.5 {
                menu_position.y += height;
                height += name.size().height;
            }

            //Outline background
            let rect_start = Point::new(position.x - ROM_OUTLINE_SIZE, position.y - ROM_OUTLINE_SIZE);
            let rect_size = Size::new(target_size.width + ROM_OUTLINE_SIZE * 2., target_size.height + height + ROM_OUTLINE_SIZE * 2.);
            piet.fill(Rect::from((rect_start, rect_size)), &MODAL_BACKGROUND.with_alpha(alpha * 0.94));

            piet.draw_text(&name, Point::new(position.x, position.y + target_size.height));

            //Help
            let text = super::get_drawable_text(piet, crate::font::FONT_SIZE, "Info", get_font_color(settings).with_alpha(alpha));
            menu_position = super::right_aligned_text(piet, menu_position, Some(Images::ButtonX), text).shift_x(-5.);

            let text = super::get_drawable_text(piet, crate::font::FONT_SIZE, "Run", get_font_color(settings).with_alpha(alpha));
            super::right_aligned_text(piet, menu_position, Some(Images::ButtonA), text);
        }

        use crate::assets::{request_asset_image, request_image, Images};

        let slot = &mut exe.boxart.borrow_mut();
        let queue = crate::get_queue_mut(&self.queue);
        if let Some(i) = request_asset_image(piet, queue, slot) {
            i.render(piet, Rect::from((position, target_size)));
        } else if let Some(i) = request_image(piet, queue, Images::Placeholder) {
            i.render(piet, Rect::from((position, target_size)));
        }
    }

    pub fn update(&mut self, state: &YaffeState) {
        if state.selected_app == self.index {
            if !self.focused {
                self.focused = true;
                self.time = ANIMATION_TIME;
            }
        } else {
            self.focused = false;
        }

        self.time = f64::max(0., self.time - state.delta_time);
    }

    pub fn get_image_size(&self, state: &YaffeState) -> Size {
        let p = state.get_platform();
        let exe = &p.apps[self.index];
        if let Ok(slot) = exe.boxart.try_borrow() {
            if let Some(i) = &slot.image {
                return i.size();
            }
        } 

        //I dont want to deal with passing Piet everywhere so we will just hardcode the placeholder size
        //Shouldn't really change
        Size::new(400., 290.)
    }
}