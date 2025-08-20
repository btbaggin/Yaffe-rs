use crate::ui::{
    get_drawable_text_with_wrap, AnimationManager, UiContainer, ModalAction, Image, Label, UiElement, WidgetId, MARGIN,
    MODAL_BACKGROUND, TITLE_SIZE,
};
use crate::{widget, Actions, DeferredAction, Graphics, LogicalPosition, Rect, ScaleFactor, YaffeState, LogicalSize};

widget!(
    pub struct InfoPane {
    y_offset: f32 = 0.,
    y_offset_max: f32 = 0.,
    container: UiContainer<(), ModalAction> = UiContainer::column(),
    offset: LogicalPosition = LogicalPosition::new(1., 0.)
}
);

// fn build_container(exe: Option<&crate::Tile>) -> Container {
//     let mut main = Container::vertical(1.);
//     if let Some(exe) = exe {
//         main.add(Label::new(&exe.name.clone(), Some(TITLE_SIZE)));

//         let mut top = Container::horizontal(0.15);
//         let mut details = Container::vertical(1.);

//         top.add(Image::new(exe.boxart.clone()));
//         for (key, value) in &exe.metadata {
//             details.add(Label::simple(&format!("{key}: {value}")));
//         }
//         top.add(details);
//         main.add(top);
//     }

//     main
// }

impl UiElement<YaffeState, DeferredAction> for InfoPane {
    fn got_focus(&mut self, state: &YaffeState, animations: &mut AnimationManager) {
        animations.animate(self, crate::offset_of!(InfoPane => offset: LogicalPosition => x), 0.).duration(0.2).start();
        self.y_offset = 0.;
        self.y_offset_max = 0.;

        // self.container = build_container(state.get_selected_tile());
    }

    fn lost_focus(&mut self, _: &YaffeState, animations: &mut AnimationManager) {
        animations.animate(self, crate::offset_of!(InfoPane => offset: LogicalPosition => x), 1.).duration(0.2).start();
    }

    fn render(&mut self, graphics: &mut Graphics, state: &YaffeState, _: &WidgetId) {
        let bounds = graphics.bounds;
        graphics.draw_rectangle(bounds, MODAL_BACKGROUND);

        let size = LogicalSize::new(0., 0.); //TODO self.container.render(graphics, &bounds);
        if let Some(app) = state.get_selected_tile() {
            let top = bounds.top() + size.y + MARGIN;
            let left = bounds.left() + MARGIN;
            //Overview
            if !app.description.is_empty() {
                let name_label = get_drawable_text_with_wrap(
                    graphics,
                    graphics.font_size(),
                    &app.description,
                    (bounds.width() - MARGIN) * graphics.scale_factor,
                );

                //If the text is too big to completely fit on screen, scroll the text after a set amount of time
                if name_label.height().to_logical(graphics) + top > bounds.height() {
                    self.y_offset_max = bounds.height() - top - name_label.height()
                }

                //Clip text so when it scrolls it wont render above the banner
                let clip = Rect::point_and_size(LogicalPosition::new(bounds.left(), top), bounds.size());
                graphics.draw_text_cropped(
                    LogicalPosition::new(left, bounds.top_left().y + self.y_offset + top),
                    clip,
                    graphics.font_color(),
                    &name_label,
                );
            }
        }
    }

    fn action(
        &mut self,
        state: &mut YaffeState,
        _: &mut AnimationManager,
        action: &Actions,
        handler: &mut DeferredAction,
    ) -> bool {
        match action {
            Actions::Back => {
                handler.revert_focus();
                true
            }
            Actions::Down => {
                self.y_offset = f32::max(
                    self.y_offset - state.settings.get_f32(crate::SettingNames::InfoScrollSpeed),
                    self.y_offset_max,
                );
                true
            }
            Actions::Up => {
                self.y_offset =
                    f32::min(self.y_offset + state.settings.get_f32(crate::SettingNames::InfoScrollSpeed), 0.);
                true
            }
            _ => false,
        }
    }
}
