use druid_shell::kurbo::{Rect, Point, Size};
use druid_shell::piet::{Piet, RenderContext, Color, FixedLinearGradient, FixedGradient, GradientStop};
use crate::{Actions};
use crate::modals::{ModalResult, ModalContent, DeferredModalAction};

#[derive(Default)]
pub struct OverlayModal {
    volume: f64,
}

impl ModalContent for OverlayModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_height(&self) -> f64 {
        32.
    }

    fn action(&mut self, action: &Actions, _: &mut DeferredModalAction) -> ModalResult {
        match action {
            Actions::Left => {
                //crate::restrictions::verify_restricted_action_with_data(state, change_volume, &mut (self.volume, -0.05));
                self.volume = f64::max(0., self.volume - 0.05);
                ModalResult::None
            }
            Actions::Right => {
                //crate::restrictions::verify_restricted_action_with_data(state, change_volume, &mut (self.volume, 0.05));
                self.volume = f64::min(1., self.volume + 0.05);
                ModalResult::None
            }
            Actions::Accept => ModalResult::Ok,
            _ => ModalResult::None
        }
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, piet: &mut Piet) {
        //TODO this doesnt work because it was loaded with the Piet of another window?
        // let image_rect = Rect::new(rect.x0, rect.y0, rect.x0 + 32., rect.y0 + 32.);
        // let i = crate::assets::request_preloaded_image(piet, crate::assets::Images::Speaker);
        //i.render(piet, image_rect);

        let rect = Rect::new(rect.x0 + 35., rect.y0, rect.x1, rect.y1);
        piet.stroke(rect, &Color::GRAY, 2.);

        let (r, g, b, _) = crate::colors::get_accent_color(settings).as_rgba();
        let stops = vec![
            GradientStop { pos: 0.0, color: Color::rgb(r, g, b), /* Color::rgb(0.07, 0.8, 1.),*/ },
            GradientStop { pos: 1., color: Color::rgb(f64::max(0., r - 0.06), f64::max(0., g - 0.5), f64::max(0., b - 0.6)),/*Color::rgb(0.01, 0.3, 0.4),*/ },
        ];

        let rect = Rect::from((rect.origin(), Size::new(rect.width() * self.volume, rect.height())));
        let radial_gradient = piet.gradient(FixedGradient::Linear(FixedLinearGradient {
            start: rect.origin(),
            end: Point::new(rect.x1, rect.y0),
            stops: stops,
        })).unwrap();

        piet.fill(rect, &radial_gradient);
    }
}