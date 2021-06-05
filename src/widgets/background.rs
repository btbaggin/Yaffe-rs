use druid_shell::kurbo::{Rect, Point, Vec2};
use druid_shell::piet::{Piet, FixedRadialGradient, FixedGradient, RenderContext, GradientStop};
use crate::{YaffeState, create_widget, colors::change_brightness};

create_widget!(Background, );
impl super::Widget for Background {
    fn render(&mut self, state: &YaffeState, rect: Rect, piet: &mut Piet) { 

        let base = crate::colors::get_accent_color(&state.settings);
        let factor = state.settings.get_f64("dark_shade_factor", &-0.6);

        let stops = vec![
            GradientStop { pos: 0.0, color: change_brightness(base, 0.), },
            GradientStop { pos: 1., color: change_brightness(base, *factor), },
        ];
        let radial_gradient = piet.gradient(FixedGradient::Radial(FixedRadialGradient {
            center: Point::new((rect.x1 - rect.x0) / 2., rect.y1),
            origin_offset: Vec2::new(100.0, 100.0),
            radius: rect.y1 + (rect.y1 * 0.1),
            stops: stops,
        })).unwrap();

        piet.fill(rect, &radial_gradient);
    }
}