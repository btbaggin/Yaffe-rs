use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::{YaffeState, widget, Actions, DeferredAction, LogicalSize, LogicalPosition, Rect, utils::Logical};
use crate::colors::*;
use crate::assets::{request_image, request_asset_image, Images};
use crate::platform::Rating;
use crate::widgets::UiElement;

widget!(pub struct InfoPane { 
    scroll_timer: f32 = 0., 
    y_offset: f32 = 0.
});
impl super::Widget for InfoPane {
    fn offset(&self) -> LogicalPosition { LogicalPosition::new(1., 0.) }

    fn got_focus(&mut self, original: Rectangle, handle: &mut DeferredAction) {
        let offset = crate::offset_of!(InfoPane => position: LogicalPosition => x);
        handle.animate_f32(self, offset, original.left() - self.layout().width(), 0.2);
        self.scroll_timer = 3.;
        self.y_offset = 0.;
    }

    fn lost_focus(&mut self, original: Rectangle, handle: &mut DeferredAction) {
        let offset = crate::offset_of!(InfoPane => position: LogicalPosition => x);
        handle.animate_f32(self, offset, original.top_left().x, 0.2);
    }

    fn render(&mut self, state: &YaffeState, rect: Rectangle, delta_time: f32, piet: &mut Graphics2D) { 
        piet.draw_rectangle(rect.clone(), MODAL_BACKGROUND);
        const IMAGE_SIZE: LogicalSize = LogicalSize::new(64., 96.);

        if let Some(app) = state.get_executable() {
            //Banner image
            let mut height = 0.;
            let mut queue = self.queue.borrow_mut();

            let slot = crate::assets::get_cached_file(&app.banner);
            let slot = &mut slot.borrow_mut();

            let mut image = request_asset_image(piet, &mut queue, slot);
            if let None = image { image = request_image(piet, &mut queue, Images::PlaceholderBanner); }
            if let Some(i) = image {
                height = (rect.width() / i.size().x as f32) * i.size().y;
                i.render(piet, Rect::point_and_size(rect.top_left().to_logical(), LogicalSize::new(rect.width() ,rect.top() + height)));
            }

            //Rating image
            let rating_image = if let crate::platform::PlatformType::Plugin = state.get_platform().kind { None }
            else {
                match app.rating {
                    Rating::Everyone => Some(Images::ErsbEveryone),
                    Rating::Everyone10 => Some(Images::ErsbEveryone10),
                    Rating::Teen => Some(Images::ErsbTeen),
                    Rating::Mature => Some(Images::ErsbMature),
                    Rating::AdultOnly => Some(Images::ErsbAdultOnly),
                    Rating::NotRated => None,
                }
            };
            
            if let Some(image) = rating_image {
                if let Some(i) = request_image(piet, &mut queue, image) {
                    //Size rating image according to banner height
                    let ratio = IMAGE_SIZE.y / height;
                    let rating_size = if ratio > 1. {
                        LogicalSize::new(IMAGE_SIZE.x / ratio, IMAGE_SIZE.y / ratio)
                    } else {
                        IMAGE_SIZE
                    };
                    i.render(piet, Rect::point_and_size(LogicalPosition::new(rect.right() - rating_size.x - crate::ui::MARGIN, rect.top() + height - rating_size.y), rating_size));
                }
            }

            //Overview
            if !app.description.is_empty() {
                let name_label = super::get_drawable_text_with_wrap(crate::font::get_title_font_size(state), &app.description, rect.width() - 10.);

                //If the text is too big to completely fit on screen, scroll the text after a set amount of time
                if name_label.height() + height > rect.height() {
                    self.scroll_timer -= delta_time;
                    if self.scroll_timer < 0. { 
                        self.y_offset -= delta_time * state.settings.get_f32(crate::SettingNames::InfoScrollSpeed);
                        self.y_offset = f32::max(self.y_offset, rect.height() - height - name_label.height()); 
                    }
                }
                
                //Clip text so when it scrolls it wont render above the banner
                //piet.save().unwrap();
                //TODO piet.clip(Rectangle::from_tuples((rect.top_left().x, rect.top_left().y + height), (rect.bottom_right().x, rect.bottom_right().y)));
                piet.draw_text(LogicalPosition::new(rect.top_left().x + crate::ui::MARGIN, rect.top_left().y + self.y_offset + height), get_font_color(&state.settings), &name_label);
                //piet.restore().unwrap();
            }
        }
    }

    fn action(&mut self, _: &mut YaffeState, action: &Actions, handler: &mut DeferredAction) -> bool {
        match action {
            Actions::Back => {
                handler.revert_focus();
                true
            }
            _ => false
        }
    }
}