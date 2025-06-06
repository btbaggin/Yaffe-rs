use crate::{YaffeState, Graphics, widget, Actions, DeferredAction, LogicalPosition, ScaleFactor, Rect};
use crate::ui::{MARGIN, get_drawable_text_with_wrap, MODAL_BACKGROUND, Container, Control, Label, Image, Widget};

const SCROLL_TIMER: f32 = 3.;

widget!(pub struct InfoPane { 
    scroll_timer: f32 = 0., 
    y_offset: f32 = 0.,
    container: Container = Container::vertical(1.),
    offset: LogicalPosition = LogicalPosition::new(1., 0.)
});

fn build_container(exe: Option<&crate::Tile>) -> Container {
    let mut main = Container::vertical(1.);
    if let Some(exe) = exe {
        main.add(Label::new(exe.name.clone(), Some(crate::ui::TITLE_SIZE)));

        let mut top = Container::horizontal(0.15);
        let mut details = Container::vertical(1.);
    
        top.add(Image::new(exe.boxart.clone()));
        for (key, value) in &exe.metadata {
            details.add(Label::simple(format!("{}: {}", key, value)));
        }
        top.add(details);
        main.add(top);
    }
    
    main
}

impl Widget for InfoPane {
    fn offset(&self) -> LogicalPosition { self.offset }

    fn got_focus(&mut self, state: &YaffeState) {
        let offset = crate::offset_of!(InfoPane => offset: LogicalPosition => x);
        self.animate(offset, 0., 0.2);
        self.scroll_timer = SCROLL_TIMER;
        self.y_offset = 0.;

        self.container = build_container(state.get_selected_tile())
    }

    fn lost_focus(&mut self, _: &YaffeState) {
        let offset = crate::offset_of!(InfoPane => offset: LogicalPosition => x);
        self.animate(offset, 1., 0.2);
    }

    fn render(&mut self, graphics: &mut Graphics, state: &YaffeState) { 
        let bounds = graphics.bounds;
        graphics.draw_rectangle(bounds, MODAL_BACKGROUND);

        let size = self.container.render(graphics, &bounds);
        if let Some(app) = state.get_selected_tile() {
   
            let top = bounds.top() + size.y + MARGIN;
            let left = bounds.left() + MARGIN;
            //Overview
            if !app.description.is_empty() {
                let name_label = get_drawable_text_with_wrap(graphics.font_size(), &app.description, (bounds.width() - MARGIN) * graphics.scale_factor);

                //If the text is too big to completely fit on screen, scroll the text after a set amount of time
                if name_label.height().to_logical(graphics) + top > bounds.height() {
                    self.scroll_timer -= graphics.delta_time;
                    if self.scroll_timer < 0. { 
                        self.y_offset -= graphics.delta_time * state.settings.get_f32(crate::SettingNames::InfoScrollSpeed);
                        self.y_offset = f32::max(self.y_offset, bounds.height() - top - name_label.height()); 
                    }
                }
                
                //Clip text so when it scrolls it wont render above the banner
                let clip = Rect::point_and_size(LogicalPosition::new(bounds.left(), top), bounds.size());
                graphics.draw_text_cropped(LogicalPosition::new(left, bounds.top_left().y + self.y_offset + top), clip, graphics.font_color(), &name_label);
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

