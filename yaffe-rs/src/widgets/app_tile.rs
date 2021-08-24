use speedy2d::Graphics2D;
use crate::Transparent;
use crate::{YaffeState, V2};
use crate::colors::*;
use crate::Rect;
use crate::widgets::Shifter;

pub const ANIMATION_TIME: f32 = 0.25;
const SELECTED_SCALAR: f32 = 0.2;
const ROM_OUTLINE_SIZE: f32 = 7.5;

const VISIBLE_FLAG: u8 = 0x01;

pub struct AppTile { 
    queue: std::sync::Arc<std::cell::RefCell<crate::JobQueue>>,
    index: usize,
    flags: u8,
    pub position: V2,
    pub size: V2,
}
impl AppTile {
    pub fn new(q: std::sync::Arc<std::cell::RefCell<crate::JobQueue>>, index: usize) -> AppTile {
        AppTile { 
            queue: q,
            index: index,
            flags: VISIBLE_FLAG,
            position: V2::new(0., 0.),
            size: V2::new(0., 0.),
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


    pub fn render(&mut self, settings: &crate::settings::SettingsFile, focused: bool, animation: f32, exe: &crate::Executable, piet: &mut Graphics2D) {
        if !self.is_visible() { return; }

        let mut target_size = self.size;
        let mut position = self.position;

        if focused {
            // let animation_remainder = (ANIMATION_TIME - self.time) / ANIMATION_TIME;
            //Have alpha fade in as the time grows to full size
            let alpha = f32::powf(animation, 2.);

            target_size = target_size * (1. + animation * SELECTED_SCALAR);
            position = position - (target_size - self.size) / 2.;

            //Position of the text and buttons for the focused game
            let mut menu_position = V2::new(position.x + target_size.x, position.y + target_size.y + 2.);

            let name = super::get_drawable_text_with_wrap(crate::font::FONT_SIZE, &exe.name, target_size.x);
            let mut height = name.height();

			//Check if we need to push the buttons below the text due to overlap
			if name.width() > target_size.x * 0.5 {
                menu_position.y += height;
                height += name.height();
            }

            //Outline background
            let rect_start = position - V2::new(ROM_OUTLINE_SIZE, ROM_OUTLINE_SIZE);
            let rect_size = V2::new(target_size.x + ROM_OUTLINE_SIZE * 2., target_size.y + height + ROM_OUTLINE_SIZE * 2.);
            piet.draw_rectangle(Rect::point_and_size(rect_start, rect_size), MODAL_BACKGROUND.with_alpha(alpha * 0.94));

            piet.draw_text(V2::new(position.x, position.y + target_size.y), get_font_color(settings).with_alpha(alpha), &name);

            //Help
            let text = super::get_drawable_text(crate::font::FONT_SIZE, "Info");
            menu_position = super::right_aligned_text(piet, menu_position, Some(Images::ButtonX), get_font_color(settings).with_alpha(alpha), text).shift_x(-5.);

            let text = super::get_drawable_text(crate::font::FONT_SIZE, "Run");
            super::right_aligned_text(piet, menu_position, Some(Images::ButtonA), get_font_color(settings).with_alpha(alpha), text);
        }

        use crate::assets::{request_asset_image, request_image, Images};

        let slot = &mut exe.boxart.borrow_mut();
        let mut queue = self.queue.borrow_mut();
        if let Some(i) = request_asset_image(piet, &mut queue, slot) {
            i.render(piet, Rect::point_and_size(position, target_size));
        } else if let Some(i) = request_image(piet, &mut queue, Images::Placeholder) {
            i.render(piet, Rect::point_and_size(position, target_size));
        }
    }

    pub fn get_image_size(&self, state: &YaffeState) -> V2 {
        let p = state.get_platform();
        let exe = &p.apps[self.index];
        if let Ok(slot) = exe.boxart.try_borrow() {
            if let Some(size) = slot.get_image_size() {
                return size;
            }
        } 

        //I dont want to deal with passing Piet everywhere so we will just hardcode the placeholder size
        //Shouldn't really change
        V2::new(400., 290.)
    }
}