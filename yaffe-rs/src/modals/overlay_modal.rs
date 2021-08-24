use speedy2d::{Graphics2D, color::Color};
use speedy2d::shape::Rectangle;
use crate::Rect;
use crate::{Actions, V2};
use crate::modals::{ModalResult, ModalContent};

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
    fn get_height(&self) -> f32 {
        32.
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        match action {
            Actions::Left => {
                match crate::platform_layer::get_and_update_volume(-VOLUME_STEP) {
                    Ok(volume) => self.volume = volume,
                    Err(e) => crate::logger::log_entry_with_message(crate::logger::LogTypes::Warning, e, "Unable to get system volume"),
                }
                ModalResult::None
            }
            Actions::Right => {
                match crate::platform_layer::get_and_update_volume(VOLUME_STEP) {
                    Ok(volume) => self.volume = volume,
                    Err(e) => crate::logger::log_entry_with_message(crate::logger::LogTypes::Warning, e, "Unable to get system volume"),
                }
                ModalResult::None
            }
            Actions::Accept => ModalResult::Ok,
            _ => ModalResult::None
        }
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rectangle, piet: &mut Graphics2D) {
        //TODO this doesnt work because it was loaded with the Piet of another window?
        // let image_rect = Rectangle::from_tuples((rect.left(), rect.top()), (rect.left() + 32., rect.top() + 32.));
        // let i = crate::assets::request_preloaded_image(piet, crate::assets::Images::Speaker);
        // i.render(piet, image_rect);

        let rect = Rectangle::from_tuples((rect.left() + 35., rect.top()), (rect.right(), rect.bottom()));
        crate::modals::modal::outline_rectangle(piet, &rect, 2., Color::GRAY);

        let accent = crate::colors::get_accent_color(settings);
        let rect = Rectangle::new(*rect.top_left(), rect.top_left() + V2::new(rect.width() * self.volume, rect.height()));

        piet.draw_rectangle(rect, accent);
    }
}