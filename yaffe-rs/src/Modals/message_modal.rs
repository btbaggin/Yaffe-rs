use crate::input::Actions;
use crate::ui::{AnimationManager, LayoutElement, UiElement, WidgetId};
use crate::modals::ModalAction;
use crate::{Graphics, LogicalSize};



//Modal for displaying a simple string
crate::widget!(
    pub struct MessageModal {
        message: String = String::new()
    }
);

impl MessageModal {
    pub fn from(message: &str) -> MessageModal {
        let mut content = MessageModal::new();
        content.message = message.to_string();
        content
    }
}
impl UiElement<(), ModalAction> for MessageModal {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize {
        let rows = self.message.len() as f32 / 80.;
        LogicalSize::new(graphics.bounds.width(), graphics.font_size() * rows)
    }

    fn action(
        &mut self,
        _state: &mut (),
        _: &mut AnimationManager,
        action: &Actions,
        handler: &mut ModalAction,
    ) -> bool {
        handler.close_if_accept(action)
    }

    fn render(&mut self, graphics: &mut Graphics, _: &(), _: &WidgetId) {
        let rect = self.layout();
        let name_label = crate::ui::get_drawable_text_with_wrap(
            graphics,
            graphics.font_size(),
            &self.message,
            rect.width() * graphics.scale_factor,
        );
        self.size.y = name_label.height();
        graphics.draw_text(*rect.top_left(), graphics.font_color(), &name_label);
    }
}
