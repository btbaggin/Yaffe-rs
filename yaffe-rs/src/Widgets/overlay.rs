use crate::assets::Images;
use crate::controls::{LABEL_SIZE, MODAL_BACKGROUND, MODAL_OVERLAY_COLOR};
use crate::logger::LogEntry;
use crate::os::{get_volume, set_volume};
use crate::ui::{image_fill, AnimationManager, DeferredAction, RightAlignment, UiElement, WidgetId, MARGIN};
use crate::{widget, Actions, Graphics, LogicalPosition, LogicalSize, OverlayState, Rect};
use speedy2d::color::Color;

const VOLUME_STEP: f32 = 0.05;

widget!(
    pub struct OverlayBackground {
        volume: f32 = 0.
    }
);
impl UiElement<OverlayState> for OverlayBackground {
    fn action(
        &mut self,
        _: &mut OverlayState,
        _: &mut AnimationManager,
        action: &Actions,
        _: &mut DeferredAction<OverlayState>,
    ) -> bool {
        match action {
            Actions::Left => {
                self.volume = f32::max(0., self.volume - VOLUME_STEP);
                set_volume(self.volume).log("Unable to get system volume");
                true
            }
            Actions::Right => {
                self.volume = f32::min(1., self.volume + VOLUME_STEP);
                set_volume(self.volume).log("Unable to get system volume");
                true
            }
            _ => false,
        }
    }

    fn render(&mut self, graphics: &mut Graphics, state: &OverlayState, _: &WidgetId) {
        let Some(ref process) = *state.process.borrow() else {
            return;
        };

        self.volume = get_volume().unwrap_or(0.);
        graphics.clear_screen(Color::TRANSPARENT);

        const WINDOW_WIDTH: f32 = 0.33;
        const WINDOW_HEIGHT: f32 = 0.25;

        // Background
        graphics.draw_rectangle(graphics.bounds, MODAL_OVERLAY_COLOR);

        // Modal
        let rect = graphics.bounds;

        let font_size = graphics.font_size();
        let image_size = LogicalSize::new(font_size, font_size);

        let size = LogicalSize::new(rect.width() * WINDOW_WIDTH, rect.height() * WINDOW_HEIGHT);
        let window_position = (rect.size() - size) / 2.;
        let window = Rect::new(window_position, window_position + size);
        graphics.draw_rectangle(window, MODAL_BACKGROUND);

        let window = Rect::from_tuples(
            (window.left() + MARGIN, window.top() + MARGIN),
            (window.right() - MARGIN, window.bottom() - MARGIN),
        );

        let volume_height = window.height() / 10.;
        let image_width = window.width() / 3.;
        let image_height = window.height() - volume_height - image_size.y - (MARGIN * 2.);

        let size = image_fill(graphics, &process.image, &LogicalSize::new(image_width, image_height));
        graphics.draw_asset_image(Rect::point_and_size(*window.top_left(), size), &process.image);

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
        let title_width = window.width() - image_width - MARGIN - text.width();
        let title =
            crate::ui::get_drawable_text_with_wrap(graphics, graphics.title_font_size(), &process.name, title_width);
        graphics.draw_text(
            LogicalPosition::new(window.left() + MARGIN + image_width, window.top()),
            graphics.font_color(),
            &title,
        );

        // Volume
        let volume_position = LogicalPosition::new(window.left(), window.top() + image_height + MARGIN);
        draw_volume_bar(
            graphics,
            volume_position,
            LogicalSize::new(window.width() - LABEL_SIZE - MARGIN, volume_height),
            self.volume,
        );

        let font_size = graphics.font_size();
        let image_size = LogicalSize::new(font_size, font_size);
        let menu = RightAlignment::new(LogicalPosition::new(window.right(), window.bottom() - font_size));
        menu.text(graphics, "Exit").image(graphics, Images::ButtonA, image_size);
    }
}

fn draw_volume_bar(graphics: &mut Graphics, position: LogicalPosition, size: LogicalSize, volume: f32) {
    graphics.simple_text(position, "Volume:");

    let position = LogicalPosition::new(position.x + LABEL_SIZE, position.y);
    //Background rectangle
    let rect = Rect::point_and_size(position, size);
    graphics.outline_rect(rect, 2., Color::GRAY);

    //Progress rectangle
    let pos = *rect.top_left();
    let size = LogicalSize::new(rect.width() * volume, rect.height());
    let rect = Rect::point_and_size(pos, size);

    graphics.draw_rectangle(rect, graphics.accent_color());
}
