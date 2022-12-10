use crate::{YaffeState, Graphics, widget, Actions, DeferredAction, LogicalSize, LogicalPosition, ScaleFactor, Rect};
use crate::assets::{request_image, request_asset_image, Images};
use crate::platform::Rating;
use crate::ui::UiElement;
use crate::ui::{MARGIN, get_font_color, get_font_size, MODAL_BACKGROUND};

widget!(pub struct InfoPane { 
    scroll_timer: f32 = 0., 
    y_offset: f32 = 0.
});
impl crate::ui::Widget for InfoPane {
    fn offset(&self) -> LogicalPosition { LogicalPosition::new(1., 0.) }

    fn got_focus(&mut self, original: Rect) {
        let offset = crate::offset_of!(InfoPane => position: LogicalPosition => x);
        self.animate(offset, original.left() - self.layout().width(), 0.2);
        self.scroll_timer = 3.;
        self.y_offset = 0.;
    }

    fn lost_focus(&mut self, original: Rect) {
        let offset = crate::offset_of!(InfoPane => position: LogicalPosition => x);
        self.animate(offset, original.top_left().x, 0.2);
    }

    fn render(&mut self, graphics: &mut Graphics, state: &YaffeState) { 
        let bounds = graphics.bounds;
        graphics.draw_rectangle(bounds.clone(), MODAL_BACKGROUND);

        if let Some(app) = state.get_executable() {
            //Banner image
            let mut top = bounds.top() + MARGIN;
            let left = bounds.left() + MARGIN;
            
            let title = crate::ui::get_drawable_text(48., &app.name);
            graphics.draw_text(LogicalPosition::new(left, top), get_font_color(&state.settings), &title);
            top += title.height();
            
            let slot = crate::assets::get_cached_file(&app.boxart);
            let slot = &mut slot.borrow_mut();
            
            let image_size = crate::ui::image_fill(graphics, slot, &Rect::percent(bounds, LogicalSize::new(0.5, 0.15)).size(), false);
            let info_height = image_size.y;
            let image = request_asset_image(graphics, slot);
            if let Some(i) = image {
                i.render(graphics, Rect::point_and_size(LogicalPosition::new(left, top), image_size));
            }

            //Rating image
            let rating_image = if let crate::platform::PlatformType::Plugin = state.get_platform().kind { None }
            else { get_rating_image(&app.rating) };
            
            if let Some(image) = rating_image {
                if let Some(i) = request_image(graphics, image) {
                    const IMAGE_SIZE: LogicalSize = LogicalSize::new(64., 96.);
                    let left = left + image_size.x + MARGIN;
                    let position = LogicalPosition::new(left, top);
                    i.render(graphics, Rect::point_and_size(position, IMAGE_SIZE));
                }
            }

            //TODO add players

            top += info_height + MARGIN;

            //Overview
            if !app.description.is_empty() {
                let name_label = crate::ui::get_drawable_text_with_wrap(get_font_size(&state.settings, graphics), &app.description, (bounds.width() - MARGIN) * graphics.scale_factor);

                //If the text is too big to completely fit on screen, scroll the text after a set amount of time
                if name_label.height().to_logical(graphics) + top > bounds.height() {
                    self.scroll_timer -= graphics.delta_time;
                    if self.scroll_timer < 0. { 
                        self.y_offset -= graphics.delta_time * state.settings.get_f32(crate::SettingNames::InfoScrollSpeed);
                        self.y_offset = f32::max(self.y_offset, bounds.height() - top - name_label.height()); 
                    }
                }
                
                //Clip text so when it scrolls it wont render above the banner
                graphics.set_clip(Some(Rect::point_and_size(LogicalPosition::new(bounds.left(), top), bounds.size())));
                graphics.draw_text(LogicalPosition::new(left, bounds.top_left().y + self.y_offset + top), get_font_color(&state.settings), &name_label);
                graphics.set_clip(None);
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

fn get_rating_image(rating: &Rating) -> Option<Images> {
    match rating {
        Rating::Everyone => Some(Images::ErsbEveryone),
        Rating::Everyone10 => Some(Images::ErsbEveryone10),
        Rating::Teen => Some(Images::ErsbTeen),
        Rating::Mature => Some(Images::ErsbMature),
        Rating::AdultOnly => Some(Images::ErsbAdultOnly),
        Rating::NotRated => None,
    }
}