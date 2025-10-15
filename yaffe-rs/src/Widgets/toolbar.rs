use crate::assets::Images;
use crate::ui::{LayoutElement, RightAlignment, WidgetId, MARGIN};
use crate::{widget, Graphics, LogicalPosition, LogicalSize, YaffeState};

widget!(
    pub struct Toolbar {}
);
impl crate::ui::UiElement<YaffeState> for Toolbar {
    fn render(&mut self, graphics: &mut Graphics, state: &YaffeState, current_focus: &WidgetId) {
        let time = chrono::Local::now();
        let rect = self.layout();

        let font_size = graphics.font_size();
        let image_size = LogicalSize::new(font_size, font_size);

        let mut alignment =
            RightAlignment::new(LogicalPosition::new(rect.right() - MARGIN, rect.bottom() - font_size - MARGIN));

        //Draw time
        let time_string = time.format("%I:%M%p");
        alignment = alignment.text(graphics, &time_string.to_string()).space();

        //Draw buttons
        //What actions we can perform depend on what's focused
        if current_focus == &crate::APP_LIST_ID {
            alignment.text(graphics, "Filter").image(graphics, Images::ButtonB, image_size).space();
        } else if current_focus == &crate::PLATFORM_LIST_ID {
            let platform = state.get_selected_group();
            if platform.kind.allow_edit() {
                alignment = alignment.text(graphics, "Settings").image(graphics, Images::ButtonX, image_size).space();
            }
            alignment.text(graphics, "Select").image(graphics, Images::ButtonA, image_size).space();
        }
    }
}
