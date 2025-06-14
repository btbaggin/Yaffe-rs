use crate::modals::{on_update_platform_close, PlatformDetailModal};
use crate::ui::{display_modal, AnimationManager, Widget, WidgetId, MARGIN, MENU_BACKGROUND, LABEL_SIZE};
use crate::os::get_and_update_volume;
use crate::widgets::AppList;
use crate::logger::LogEntry;
use crate::{
    state::GroupType, widget, Actions, DeferredAction, LogicalPosition, LogicalSize, Rect, ScaleFactor, YaffeState,
};

use speedy2d::color::Color;

const VOLUME_STEP: f32 = 0.05;

widget!(
    pub struct OverlayBackground {
        volume: f32 = 0.
    }
);
impl Widget for OverlayBackground {
    fn got_focus(&mut self, state: &YaffeState, animations: &mut AnimationManager) {
        self.volume = get_and_update_volume(0.).unwrap_or(0.);

    }
    fn action(
        &mut self,
        state: &mut YaffeState,
        _: &mut AnimationManager,
        action: &Actions,
        handler: &mut DeferredAction,
    ) -> bool {
        match action {
            Actions::Left => {
                self.volume = get_and_update_volume(-VOLUME_STEP).log("Unable to get system volume");
                true
            }
            Actions::Right => {
                self.volume = get_and_update_volume(VOLUME_STEP).log("Unable to get system volume");
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, graphics: &mut crate::Graphics, state: &YaffeState, current_focus: &WidgetId) {
        let rect = Rect::new(LogicalPosition::new(0., 0.), LogicalPosition::new(100., 100.));
        graphics.simple_text(*rect.top_left(), "Volume:");

        //Background rectangle
        let rect = Rect::from_tuples((rect.left() + LABEL_SIZE, rect.top()), (rect.right(), rect.bottom()));
        crate::ui::outline_rectangle(graphics, &rect, 2., Color::GRAY);

        //Progress rectangle
        let accent = graphics.accent_color();
        let rect = Rect::percent(rect, LogicalSize::new(self.volume, 1.));

        graphics.draw_rectangle(rect, accent);
    }
}
