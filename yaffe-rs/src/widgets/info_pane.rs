use crate::{YaffeState, Graphics, widget, ui::MARGIN, Actions, DeferredAction, LogicalSize, LogicalPosition, ScaleFactor, Rect};
use crate::colors::*;
use crate::assets::{request_image, request_asset_image, Images};
use crate::platform::Rating;
use crate::widgets::UiElement;
use crate::logger::PanicLogEntry;

widget!(pub struct InfoPane { 
    scroll_timer: f32 = 0., 
    y_offset: f32 = 0.
});
impl super::Widget for InfoPane {
    fn offset(&self) -> LogicalPosition { LogicalPosition::new(1., 0.) }

    fn got_focus(&mut self, original: Rect, handle: &mut DeferredAction) {
        let offset = crate::offset_of!(InfoPane => position: LogicalPosition => x);
        handle.animate_f32(self, offset, original.left() - self.layout().width(), 0.2);
        self.scroll_timer = 3.;
        self.y_offset = 0.;
    }

    fn lost_focus(&mut self, original: Rect, handle: &mut DeferredAction) {
        let offset = crate::offset_of!(InfoPane => position: LogicalPosition => x);
        handle.animate_f32(self, offset, original.top_left().x, 0.2);
    }

    fn render(&mut self, graphics: &mut Graphics, state: &YaffeState) { 
        let bounds = graphics.bounds;
        graphics.draw_rectangle(bounds.clone(), MODAL_BACKGROUND);
        const IMAGE_SIZE: LogicalSize = LogicalSize::new(64., 96.);

        if let Some(app) = state.get_executable() {
            //Banner image
            let mut height = 0.;
            let lock = self.queue.lock().log_and_panic();
            let mut queue = lock.borrow_mut();

            let slot = crate::assets::get_cached_file(&app.banner);
            let slot = &mut slot.borrow_mut();

            let mut image = request_asset_image(graphics, &mut queue, slot);
            if let None = image { image = request_image(graphics, &mut queue, Images::PlaceholderBanner); }
            if let Some(i) = image {
                //Invalid image files can cause size to be zero
                let size = i.size().to_logical(graphics.scale_factor);
                if size.x > 0. {
                    height = (bounds.width() / size.x as f32) * size.y;
                    i.render(graphics, Rect::point_and_size(*bounds.top_left(), LogicalSize::new(bounds.width(), bounds.top() + height)));
                }
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
                if let Some(i) = request_image(graphics, &mut queue, image) {
                    //Size rating image according to banner height
                    let ratio = IMAGE_SIZE.y / height;
                    let rating_size = if ratio > 1. {
                        LogicalSize::new(IMAGE_SIZE.x / ratio, IMAGE_SIZE.y / ratio)
                    } else {
                        IMAGE_SIZE
                    };
                    let position = LogicalPosition::new(bounds.right() - rating_size.x - crate::ui::MARGIN, bounds.top() + height - rating_size.y);
                    i.render(graphics, Rect::point_and_size(position, rating_size));
                }
            }

            //Overview
            if !app.description.is_empty() {
                let name_label = super::get_drawable_text_with_wrap(crate::font::get_font_size(&state.settings, graphics), &app.description, (bounds.width() - MARGIN) * graphics.scale_factor);

                //If the text is too big to completely fit on screen, scroll the text after a set amount of time
                if name_label.height().to_logical(graphics) + height > bounds.height() {
                    self.scroll_timer -= graphics.delta_time;
                    if self.scroll_timer < 0. { 
                        self.y_offset -= graphics.delta_time * state.settings.get_f32(crate::SettingNames::InfoScrollSpeed);
                        self.y_offset = f32::max(self.y_offset, bounds.height() - height - name_label.height()); 
                    }
                }
                
                //Clip text so when it scrolls it wont render above the banner
                //piet.save().unwrap();
                //TODO piet.clip(Rectangle::from_tuples((rect.top_left().x, rect.top_left().y + height), (rect.bottom_right().x, rect.bottom_right().y)));
                graphics.draw_text(LogicalPosition::new(bounds.top_left().x + crate::ui::MARGIN, bounds.top_left().y + self.y_offset + height), get_font_color(&state.settings), &name_label);
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