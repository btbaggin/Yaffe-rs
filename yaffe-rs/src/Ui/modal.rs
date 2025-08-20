use crate::assets::Images;
use crate::ui::controls::{change_brightness, MARGIN, MODAL_BACKGROUND, MODAL_OVERLAY_COLOR};
use crate::ui::{RightAlignment, UiElement, AnimationManager, WidgetId, UiContainer, ContainerSize, LayoutElement};
use crate::{Actions, LogicalPosition, LogicalSize, Rect, YaffeState, Graphics};
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
    close: Option<bool>
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
            _ => false
        }
    }
}

pub type ModalContent = dyn UiElement<(), ModalAction>;

pub type ModalOnClose = fn(&mut YaffeState, bool, &ModalContent);
pub struct Modal {
    confirmation_button: Option<String>,
    content: Box<UiContainer<(), ModalAction>>,
    on_close: Option<ModalOnClose>,
}
impl Modal {
    pub fn overlay(content: impl UiElement<(), ModalAction> + 'static) -> Modal {
        let mut control = UiContainer::column();
        control.background_color(MODAL_BACKGROUND)
               .add_child(content, ContainerSize::Fixed(36.));
        Modal {
            confirmation_button: Some(String::from("Exit")),
            content: Box::new(control),
            on_close: None
        }
    }

    // pub fn action(&mut self, action: &crate::Actions, helper: &mut ()) -> ModalResult {
    //     self.content.action(action, helper)
    // }
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
        let titlebar = 
            Rect::point_and_size(*layout.top_left(), layout.size() - LogicalSize::new(PADDING * 2., PADDING));
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

fn build_modal(title: String, confirmation_button: Option<String>, content: impl UiElement<(), ModalAction> + 'static) -> UiContainer<(), ModalAction> {
    let mut control = UiContainer::column();
    control.background_color(MODAL_BACKGROUND)
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
    on_close: Option<ModalOnClose>,
) {
    let confirm = confirmation_button.map(String::from);

    let content = build_modal(String::from(title), confirm.clone(), content);
    let m = Modal {
        confirmation_button: confirm,
        content: Box::new(content),
        on_close
    };

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
    const PADDING: f32 = 2.;
    let titlebar_size = graphics.title_font_size();
    let toolbar_size = graphics.font_size() + MARGIN;

    let padding = LogicalSize::new(graphics.bounds.width() * 0.1, graphics.bounds.height() * 0.1);
    let rect = Rect::new(*graphics.bounds.top_left() + padding, graphics.bounds.size() - padding);
    let width = match ModalSize::Half {
        ModalSize::Third => graphics.bounds.width() * 0.33,
        ModalSize::Half => graphics.bounds.width() * 0.5,
        ModalSize::Full => graphics.bounds.width(),
    };
    let content_size = LogicalSize::new(graphics.bounds.height(), width);

    // //Calulate size
    let mut size = LogicalSize::new(MARGIN * 2. + content_size.x, MARGIN * 2. + titlebar_size + content_size.y);
    if modal.confirmation_button.is_some() {
        size.y += toolbar_size;
    }

    let window_position = (rect.size() - size) / 2. + padding;
    let window = Rect::new(window_position, window_position + size);

    //Background
    graphics.draw_rectangle(graphics.bounds, MODAL_OVERLAY_COLOR);

    // //Content
    // //Window + margin for window + margin for icon
    // let content_pos = LogicalPosition::new(
    //     window_position.x + MARGIN + PADDING,
    //     window_position.y + MARGIN + titlebar_size + PADDING,
    // );
    graphics.bounds = window;//Rect::new(content_pos, content_pos + content_size);
    modal.content.render(graphics, &(), &modal.content.id());
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
