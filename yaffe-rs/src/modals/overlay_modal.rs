use speedy2d::color::Color;
use crate::Rect;
use crate::{Actions, LogicalSize};
use crate::modals::{ModalResult, ModalContent, modal_width, ModalSize};
use crate::logger::LogEntry;
use crate::ui_control::{get_accent_color, LABEL_SIZE};

const VOLUME_STEP: f32 = 0.05;

pub struct OverlayModal {
    volume: f32,
}

impl OverlayModal {
    pub fn new() -> OverlayModal {
        let volume = match crate::platform_layer::get_and_update_volume(0.) {
            Ok(volume) => volume,
            Err(_) => 0.,
        };
        OverlayModal { volume }
    }
}

impl ModalContent for OverlayModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn size(&self, _: &crate::settings::SettingsFile, rect: Rect, graphics: &crate::Graphics) -> LogicalSize {
        let height = 32. * graphics.scale_factor;
        LogicalSize::new(modal_width(rect, ModalSize::Half), height)
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        match action {
            Actions::Left => {
                self.volume = crate::platform_layer::get_and_update_volume(-VOLUME_STEP).log("Unable to get system volume");
                ModalResult::None
            },
            Actions::Right => {
                self.volume = crate::platform_layer::get_and_update_volume(VOLUME_STEP).log("Unable to get system volume");
                ModalResult::None
            },
            Actions::Accept => ModalResult::Ok,
            _ => ModalResult::None
        }
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        graphics.simple_text(*rect.top_left(), settings, "Volume:"); 

        //Background rectangle
        let rect = Rect::from_tuples((rect.left() + LABEL_SIZE, rect.top()), (rect.right(), rect.bottom()));
        crate::modals::outline_rectangle(graphics, &rect, 2., Color::GRAY);

        //Progress rectangle
        let accent = get_accent_color(settings);
        let rect = Rect::percent(rect, LogicalSize::new(self.volume, 1.));

        graphics.draw_rectangle(rect, accent);
    }
}