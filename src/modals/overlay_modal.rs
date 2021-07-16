use speedy2d::{Graphics2D, color::Color};
use speedy2d::shape::Rectangle;
use crate::Rect;
use crate::{Actions, V2};
use crate::modals::{ModalResult, ModalContent};

#[derive(Default)]
pub struct OverlayModal {
    volume: f32,
}

impl ModalContent for OverlayModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_height(&self) -> f32 {
        32.
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        match action {
            Actions::Left => {
                //crate::restrictions::verify_restricted_action_with_data(state, change_volume, &mut (self.volume, -0.05));
                self.volume = f32::max(0., self.volume - 0.05);
                ModalResult::None
            }
            Actions::Right => {
                //crate::restrictions::verify_restricted_action_with_data(state, change_volume, &mut (self.volume, 0.05));
                self.volume = f32::min(1., self.volume + 0.05);
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
        // let stops = vec![
        //     GradientStop { pos: 0.0, color: Color::rgb(r, g, b), /* Color::rgb(0.07, 0.8, 1.),*/ },
        //     GradientStop { pos: 1., color: Color::rgb(f64::max(0., r - 0.06), f64::max(0., g - 0.5), f64::max(0., b - 0.6)),/*Color::rgb(0.01, 0.3, 0.4),*/ },
        // ];

        let rect = Rectangle::new(*rect.top_left(), rect.top_left() + V2::new(rect.width() * self.volume, rect.height()));
        // let radial_gradient = piet.gradient(FixedGradient::Linear(FixedLinearGradient {
        //     start: rect.origin(),
        //     end: v2::new(rect.x1, rect.y0),
        //     stops: stops,
        // })).unwrap();

        piet.draw_rectangle(rect, accent);
    }
}