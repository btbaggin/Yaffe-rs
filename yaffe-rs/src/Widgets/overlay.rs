use crate::logger::LogEntry;
use crate::os::get_and_update_volume;
use crate::ui::{AnimationManager, Widget, WidgetId, LABEL_SIZE, MARGIN, MODAL_BACKGROUND, MODAL_OVERLAY_COLOR};
use crate::{widget, Actions, Graphics, LogicalPosition, LogicalSize, OverlayState, Rect};
use speedy2d::color::Color;

const VOLUME_STEP: f32 = 0.05;

widget!(
    pub struct OverlayBackground {
        volume: f32 = get_and_update_volume(0.).unwrap_or(0.)
    }
);
impl Widget<OverlayState, ()> for OverlayBackground {
    fn action(&mut self, _: &mut OverlayState, _: &mut AnimationManager, action: &Actions, _: &mut ()) -> bool {
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

    fn render(&mut self, graphics: &mut Graphics, state: &OverlayState, _: &WidgetId) {
        let Some(ref process) = *state.process.borrow() else {
            return;
        };

        graphics.clear_screen(Color::TRANSPARENT);

        const WINDOW_WIDTH: f32 = 0.33;
        const WINDOW_HEIGHT: f32 = 0.25;

        // Background
        graphics.draw_rectangle(graphics.bounds, MODAL_OVERLAY_COLOR);

        // Modal
        let rect = graphics.bounds;

        let size = LogicalSize::new(rect.width() * WINDOW_WIDTH, rect.height() * WINDOW_HEIGHT);
        let window_position = (rect.size() - size) / 2.;
        let window = Rect::new(window_position, window_position + size);
        graphics.draw_rectangle(window, MODAL_BACKGROUND);

        let window = Rect::from_tuples(
            (window.left() + MARGIN, window.top() + MARGIN),
            (window.right() - MARGIN, window.bottom() - MARGIN),
        );

        //Draw time
        let time = chrono::Local::now();
        let time_string = time.format("%I:%M%p");
        let text = crate::ui::get_drawable_text(graphics, graphics.title_font_size(), &time_string.to_string());
        graphics.draw_text(
            LogicalPosition::new(window.right() - text.width(), window.top()),
            graphics.font_color(),
            &text,
        );

        // Draw Title
        let title = crate::ui::get_drawable_text(graphics, graphics.title_font_size(), &process.name);
        graphics.draw_text(*window.top_left(), graphics.font_color(), &title);

        // TODO draw image, this required processing events on the overlay window
        graphics
            .draw_asset_image(Rect::point_and_size(*window.top_left(), LogicalSize::new(100., 100.)), &process.image);

        // Volume
        let volume_position = LogicalPosition::new(window.left(), window.top() + (window.height() / 2.));
        draw_volume_bar(
            graphics,
            volume_position,
            LogicalSize::new(window.width() - LABEL_SIZE - MARGIN, window.height() / 10.),
            self.volume,
        );
    }
}

fn draw_volume_bar(graphics: &mut Graphics, position: LogicalPosition, size: LogicalSize, volume: f32) {
    graphics.simple_text(position, "Volume:");

    let position = LogicalPosition::new(position.x + LABEL_SIZE, position.y);
    //Background rectangle
    let rect = Rect::point_and_size(position, size);
    crate::ui::outline_rectangle(graphics, &rect, 2., Color::GRAY);

    //Progress rectangle
    let pos = *rect.top_left();
    let size = LogicalSize::new(rect.width() * volume, rect.height());
    let rect = Rect::point_and_size(pos, size);

    graphics.draw_rectangle(rect, graphics.accent_color());
}
