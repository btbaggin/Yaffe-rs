use crate::assets::Images;
use crate::ui::{WidgetId, MARGIN, RightAlignment};
use crate::{widget, LogicalPosition, YaffeState, Graphics, LogicalSize, DeferredAction};

widget!(
    pub struct Toolbar { }
);
impl crate::ui::Widget<YaffeState, DeferredAction> for Toolbar {
    fn render(&mut self, graphics: &mut Graphics, state: &YaffeState, current_focus: &WidgetId) {
        let time = chrono::Local::now();
        let rect = graphics.bounds;

        let font_size = graphics.font_size();
        let image_size = LogicalSize::new(font_size, font_size);
        
        let mut alignment = RightAlignment::new(LogicalPosition::new(rect.right() - MARGIN, rect.bottom() - font_size - MARGIN));

        //Draw time
        let time_string = time.format("%I:%M%p");
        alignment = alignment.text(graphics, &time_string.to_string()).space();

        //Draw buttons
        //What actions we can perform depend on what's focused
        if current_focus.is_focused::<crate::widgets::AppList>() {
            alignment
                .text(graphics, "Filter")
                .image(graphics, Images::ButtonB, image_size)
                .space();
        } else if current_focus.is_focused::<crate::widgets::PlatformList>() {
            let platform = state.get_selected_group();
            if platform.kind.allow_edit() {
                alignment = alignment
                    .text(graphics, "Settings")
                    .image(graphics, Images::ButtonX, image_size)
                    .space();
            }
            alignment
                .text(graphics, "Select")
                .image(graphics, Images::ButtonA, image_size)
                .space();
        }
    }
}
