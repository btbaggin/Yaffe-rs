use crate::assets::Images;
use crate::ui::controls::{change_brightness, MARGIN, MODAL_BACKGROUND, MODAL_OVERLAY_COLOR};
use crate::ui::{AnimationManager, ContainerSize, LayoutElement, RightAlignment, UiContainer, UiElement, WidgetId, Justification};
use crate::{Actions, Graphics, LogicalPosition, LogicalSize, Rect, YaffeState};
use std::collections::HashMap;

#[allow(dead_code)]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ModalSize {
    Third,
    Half,
    Full,
}

pub struct ModalAction {
    close: Option<bool>,
}
impl ModalAction {
    pub fn close_if_accept(&mut self, action: &Actions) -> bool {
        match action {
            Actions::Accept => {
                self.close = Some(true);
                true
            }
            Actions::Back => {
                self.close = Some(false);
                true
            }
            _ => false,
        }
    }
}

pub type ModalContent = dyn UiElement<(), ModalAction>;

pub type ModalOnClose = fn(&mut YaffeState, bool, &ModalContent);
pub struct Modal {
    content: Box<UiContainer<(), ModalAction>>,
    on_close: Option<ModalOnClose>,
    width: ModalSize,
}

crate::widget!(
    pub struct ModalTitlebar {
        title: String = String::new()
    }
);
impl ModalTitlebar {
    pub fn from(title: String) -> ModalTitlebar {
        let mut titlebar = ModalTitlebar::new();
        titlebar.title = title;
        titlebar
    }
}
impl crate::ui::UiElement<(), ModalAction> for ModalTitlebar {
    fn render(&mut self, graphics: &mut Graphics, _: &(), _: &WidgetId) {
        let layout = self.layout();
        const PADDING: f32 = 2.;
        let titlebar_color = graphics.accent_color();
        let titlebar_color = change_brightness(&titlebar_color, graphics.light_shade_factor());

        let pos = *layout.top_left();
        let pos = LogicalPosition::new(pos.x + PADDING, pos.y + PADDING);
        let titlebar = Rect::point_and_size(pos, layout.size() - LogicalSize::new(PADDING * 2., PADDING));
        graphics.draw_rectangle(titlebar, titlebar_color);

        let title_text = crate::ui::get_drawable_text(graphics, layout.height(), &self.title);
        let title_pos = *layout.top_left() + LogicalPosition::new(MARGIN + PADDING, PADDING);
        graphics.draw_text(title_pos, graphics.font_color(), &title_text);
    }
}

// crate::widget!(
//     pub struct ModalContent {
//         content: ModalContent
//     }
// );
// impl crate::ui::UiElement<(), ()> for ModalContent {
//     fn render(&mut self, graphics: &mut Graphics, state: &(), current_focus: &WidgetId) {

//     }
// }

crate::widget!(
    pub struct ModalToolbar {
        confirmation_button: String = String::new()
    }
);
impl ModalToolbar {
    pub fn from(confirm: String) -> ModalToolbar {
        let mut content = ModalToolbar::new();
        content.confirmation_button = confirm;
        content
    }
}
impl crate::ui::UiElement<(), ModalAction> for ModalToolbar {
    fn render(&mut self, graphics: &mut Graphics, _: &(), _: &WidgetId) {
        let rect = self.layout();

        let right = LogicalPosition::new(rect.right() - MARGIN, rect.top());
        let image_size = LogicalSize::new(graphics.font_size(), graphics.font_size());
        let mut alignment = RightAlignment::new(right);
        for t in [("Cancel", Images::ButtonB), (&self.confirmation_button[..], Images::ButtonA)] {
            alignment = alignment.text(graphics, t.0).image(graphics, t.1, image_size).space();
        }
    }
}

fn build_modal(
    title: String,
    confirmation_button: Option<String>,
    content: impl UiElement<(), ModalAction> + 'static,
) -> UiContainer<(), ModalAction> {
    let mut control = UiContainer::column();
    control
        .background_color(MODAL_BACKGROUND)
        .justify(Justification::Center)
        .add_child(ModalTitlebar::from(title), ContainerSize::Fixed(36.))
        .add_child(content, ContainerSize::Shrink);

    if let Some(confirm) = confirmation_button {
        control.add_child(ModalToolbar::from(confirm), ContainerSize::Fixed(24.));
    }
    control
}

pub fn display_modal(
    state: &mut YaffeState,
    title: &str,
    confirmation_button: Option<&str>,
    content: impl UiElement<(), ModalAction> + 'static,
    width: ModalSize,
    on_close: Option<ModalOnClose>,
) {
    let confirm = confirmation_button.map(String::from);

    let content = build_modal(String::from(title), confirm, content);
    let m = Modal { content: Box::new(content), on_close, width };

    let mut modals = state.modals.lock().unwrap();
    modals.push(m);
}

pub fn update_modal(state: &mut YaffeState, animations: &mut AnimationManager, action: &Actions) {
    //This method can call into display_modal above, which locks the mutex
    //If we lock here that call will wait infinitely
    //We can get_mut here to ensure compile time exclusivity instead of locking
    //That allows us to call display_modal in close() below
    let modals = state.modals.get_mut().unwrap();
    if let Some(modal) = modals.last_mut() {
        let mut h = ModalAction { close: None };
        modal.content.action(&mut (), animations, action, &mut h);

        if let Some(accept) = h.close {
            let modal = modals.pop().unwrap();
            if let Some(close) = modal.on_close {
                // Content will always be second (after title, before buttons)
                let content = modal.content.get_child(1).as_ref();
                close(state, accept, content);
            }
        }
    }
}

pub fn is_modal_open(state: &YaffeState) -> bool {
    let modals = state.modals.lock().unwrap();
    !modals.is_empty()
}

/// Renders a modal window along with its contents
pub fn render_modal(modal: &mut Modal, graphics: &mut crate::Graphics) {
    let rect = graphics.bounds;
    let width = match modal.width {
        ModalSize::Third => graphics.bounds.width() * 0.33,
        ModalSize::Half => graphics.bounds.width() * 0.5,
        ModalSize::Full => graphics.bounds.width(),
    };
    let content_size = LogicalSize::new(width, graphics.bounds.height());
    let window_position = LogicalPosition::new((rect.width() - width) / 2., rect.top());

    //Background
    graphics.draw_rectangle(graphics.bounds, MODAL_OVERLAY_COLOR);

    // TODO need to have some margin around content. Try to make content another trait again
    graphics.bounds = Rect::point_and_size(window_position, content_size);
    modal.content.render(graphics, &(), &modal.content.get_id());
}

pub fn render_toasts(toasts: &HashMap<u64, String>, graphics: &mut crate::Graphics) {
    let count = toasts.len();
    let x = (graphics.bounds.right() + graphics.bounds.left()) / 2.;
    let y = graphics.bounds.bottom();

    const WIDTH: f32 = 500.;
    let font_size = graphics.font_size();
    let height = font_size * count as f32 + MARGIN * 2.;

    let rect = Rect::point_and_size(LogicalPosition::new(x - WIDTH / 2., y - height), LogicalSize::new(WIDTH, height));
    graphics.draw_rectangle(rect, MODAL_BACKGROUND);

    let mut curr_y = rect.top() + MARGIN;
    for toast in toasts.values() {
        let text = crate::ui::get_drawable_text(graphics, graphics.font_size(), toast);
        graphics.draw_text(LogicalPosition::new(x - text.width() / 2., curr_y), graphics.font_color(), &text);
        curr_y += font_size;
    }
}
