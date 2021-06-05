use druid_shell::kurbo::{Rect, Point, Size};
use druid_shell::piet::{Piet, RenderContext, TextLayout};
use crate::{YaffeState, create_widget, Actions, DeferredAction};
use crate::colors::*;
use crate::assets::{request_image, request_asset_image, Images};
use crate::platform::Rating;

create_widget!(InfoPane, scroll_timer: f64 = 0., y_offset: f64 = 0.);
impl super::Widget for InfoPane {
    fn layout(&self, space: &Rect, size: Size) -> Rect { 
        let position = Point::new(space.x0 + space.width(), space.y0);
        Rect::from((position, size))
    }

    fn got_focus(&mut self, layout: &Rect, handle: &mut DeferredAction) {
        handle.animate(self, Point::new(layout.x0 - layout.width(), layout.y0), 0.2);
        self.scroll_timer = 3.;
        self.y_offset = 0.;
    }

    fn lost_focus(&mut self, layout: &Rect, handle: &mut DeferredAction) {
        handle.animate(self, Point::new(layout.x0, layout.y0), 0.2);
    }

    fn render(&mut self, state: &YaffeState, rect: Rect, piet: &mut Piet) { 
        piet.fill(rect, &MODAL_BACKGROUND);
        const IMAGE_SIZE: Size = Size::new(64., 96.);

        if let Some(app) = state.get_executable() {
            //Banner image
            let mut height = 0.;
            let queue = crate::get_queue_mut(&self.queue);
            let slot = &mut app.banner.borrow_mut();
            let mut image = request_asset_image(piet, queue, slot);
            if let None = image { image = request_image(piet, queue, Images::PlaceholderBanner); }
            if let Some(i) = image {
                height = (rect.width() / i.size().width) * i.size().height;
                i.render(piet, Rect::new(rect.x0, rect.y0, rect.x1 ,rect.y0 + height));
            }

            //Rating image
            let rating_image = match app.rating {
                Rating::Everyone => Some(Images::ErsbEveryone),
                Rating::Everyone10 => Some(Images::ErsbEveryone10),
                Rating::Teen => Some(Images::ErsbTeen),
                Rating::Mature => Some(Images::ErsbMature),
                Rating::AdultOnly => Some(Images::ErsbAdultOnly),
                Rating::NotRated => None,
            };
            if let Some(image) = rating_image {
                if let Some(i) = request_image(piet, queue, image) {
                    //Size rating image according to banner height
                    let ratio = IMAGE_SIZE.height / height;
                    let rating_size = if ratio > 1. {
                        Size::new(IMAGE_SIZE.width / ratio, IMAGE_SIZE.height / ratio)
                    } else {
                        IMAGE_SIZE
                    };
                    i.render(piet, Rect::from((Point::new(rect.x1 - rating_size.width - crate::ui::MARGIN, rect.y0 + height - rating_size.height), rating_size)));
                }
            }

            //Overview
            if !app.overview.is_empty() {
                let name_label = super::get_drawable_text_with_wrap(piet, crate::font::get_info_font_size(state), &app.overview, get_font_color(&state.settings), rect.width() - 10.);

                //If the text is too big to completely fit on screen, scroll the text after a set amount of time
                if name_label.size().height + height > rect.height() {
                    self.scroll_timer -= state.delta_time;
                    if self.scroll_timer < 0. { 
                        self.y_offset -= state.delta_time * state.settings.get_f64("info_scroll_speed", &20.); 
                        self.y_offset = f64::max(self.y_offset, rect.height() - height - name_label.size().height); 
                    }
                }
                
                //Clip text so when it scrolls it wont render above the banner
                piet.save().unwrap();
                piet.clip(Rect::new(rect.x0, rect.y0 + height, rect.x1, rect.y1));
                piet.draw_text(&name_label, Point::new(rect.x0 + crate::ui::MARGIN, rect.y0 + self.y_offset + height));
                piet.restore().unwrap();
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