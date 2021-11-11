use speedy2d::{Graphics2D, color::Color};
use speedy2d::shape::Rectangle;
use crate::Rect;
use crate::{Actions, V2};
use crate::modals::{ModalResult, ModalContent};
use crate::logger::LogEntry;

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
    fn get_height(&self, _: f32) -> f32 {
        32.
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

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rectangle, piet: &mut Graphics2D) {
        let label = crate::widgets::get_drawable_text(crate::font::FONT_SIZE, "Volume:");
        piet.draw_text(*rect.top_left(), crate::modals::get_font_color(settings), &label); 

        //Background rectangle
        let rect = Rectangle::from_tuples((rect.left() + crate::ui::LABEL_SIZE, rect.top()), (rect.right(), rect.bottom()));
        crate::modals::outline_rectangle(piet, &rect, 2., Color::GRAY);

        //Progress rectangle
        let accent = crate::colors::get_accent_color(settings);
        let rect = Rectangle::new(*rect.top_left(), rect.top_left() + V2::new(rect.width() * self.volume, rect.height()));

        piet.draw_rectangle(rect, accent);
    }
}