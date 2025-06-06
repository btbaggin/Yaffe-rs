use speedy2d::color::Color;
use crate::Rect;
use crate::{Actions, LogicalSize};
use crate::modals::{ModalResult, ModalContent, ModalSize};
use crate::logger::LogEntry;
use crate::os::get_and_update_volume;
use crate::ui::LABEL_SIZE;

const VOLUME_STEP: f32 = 0.05;

pub struct OverlayModal {
    volume: f32,
}

impl OverlayModal {
    pub fn new() -> OverlayModal {
        let volume = get_and_update_volume(0.).unwrap_or(0.);
        OverlayModal { volume }
    }
}

impl ModalContent for OverlayModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, rect: Rect, graphics: &crate::Graphics) -> LogicalSize {
        let height = 32. * graphics.scale_factor;
        LogicalSize::new(Self::modal_width(rect, ModalSize::Half), height)
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        match action {
            Actions::Left => {
                self.volume = get_and_update_volume(-VOLUME_STEP).log("Unable to get system volume");
                ModalResult::None
            },
            Actions::Right => {
                self.volume = get_and_update_volume(VOLUME_STEP).log("Unable to get system volume");
                ModalResult::None
            },
            Actions::Accept => ModalResult::Ok,
            _ => ModalResult::None
        }
    }

    fn render(&self, rect: Rect, graphics: &mut crate::Graphics) {
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