use crate::controls::{MODAL_BACKGROUND, MODAL_OVERLAY_COLOR};
use crate::ui::{
    AnimationManager, ContainerSize, DeferredAction, DeferredActionTrait, Justification,
    LayoutElement, UiContainer, UiElement, WidgetTree, MARGIN,
};
use crate::{Actions, LogicalPosition, LogicalSize, Rect};
use std::collections::HashMap;
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

mod info_modal;
mod list_modal;
mod message_modal;
mod platform_detail_modal;
mod restricted_modal;
mod scraper_modal;
mod settings_modal;
mod modal_content;
mod modal_deferred_actions;

pub use info_modal::InfoModal;
pub use list_modal::ListModal;
pub use message_modal::MessageModal;
pub use platform_detail_modal::{on_add_platform_close, on_update_platform_close, PlatformDetailModal};
pub use restricted_modal::{on_restricted_modal_close, verify_restricted_action, RestrictedMode, SetRestrictedModal};
pub use scraper_modal::{on_game_found_close, on_platform_found_close, ScraperModal};
pub use settings_modal::{on_settings_close, SettingsModal};
pub use modal_content::ModalContentElement;
pub use modal_deferred_actions::{ModalClose, DisplayModal};

use modal_content::{ModalToolbar, ModalTitlebar};

pub struct Toast {
    message: String,
    time: f32,
}
impl Toast {
    pub fn new(message: &str, time: f32) -> Toast {
        Toast { message: message.to_string(), time }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum ModalSize {
    Third,
    Half,
    Full,
}

pub trait ModalInputHandler<T> {
    fn as_any(&self) -> &dyn std::any::Any;
    fn action(
        &mut self,
        _state: &mut T,
        _animations: &mut AnimationManager,
        _action: &Actions,
        _handler: &mut DeferredAction<T>,
        _container: &mut UiContainer<T>,
    ) -> bool {
        false
    }
}

pub type ModalOnClose<T> = fn(&mut T, bool, &ModalContentElement<T>, &mut DeferredAction<T>);
pub struct Modal<T: 'static> {
    content: Box<UiContainer<T>>,
    on_close: Option<ModalOnClose<T>>,
    width: ModalSize,
}
impl<T: 'static> Deref for Modal<T> {
    type Target = UiContainer<T>;
    fn deref(&self) -> &Self::Target { &self.content }
}
impl<T: 'static> DerefMut for Modal<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.content }
}

fn build_modal<T>(
    title: String,
    confirmation_button: Option<String>,
    content: ModalContentElement<T>,
) -> UiContainer<T> {
    let mut control = UiContainer::column();
    control
        .background_color(MODAL_BACKGROUND)
        .justify(Justification::Center)
        .add_child(ModalTitlebar::from(title), ContainerSize::Fixed(36.))
        .add_child(content, ContainerSize::Shrink);

    if let Some(confirm) = confirmation_button {
        control.add_child(ModalToolbar::<T>::from(confirm), ContainerSize::Fixed(24.));
    }
    control
}

pub fn display_error<T>(ui: &mut WidgetTree<T>, message: String) {
    let message = MessageModal::from(&message);
    display_modal_raw(ui, "Error", None, message, ModalSize::Half, None);
}

pub fn display_modal_raw<T>(
    ui: &mut WidgetTree<T>,
    title: &str,
    confirmation_button: Option<&str>,
    content: ModalContentElement<T>,
    width: ModalSize,
    on_close: Option<ModalOnClose<T>>,
) {
    let confirm = confirmation_button.map(String::from);

    let content = build_modal(String::from(title), confirm, content);
    let m = Modal { content: Box::new(content), on_close, width };

    let mut modals = ui.modals.lock().unwrap();
    modals.push(m);
}

pub fn update_modal<T>(
    ui: &mut WidgetTree<T>,
    animations: &mut AnimationManager,
    action: &Actions,
    handler: &mut DeferredAction<T>,
) -> bool {
    let modals = ui.modals.get_mut().unwrap();
    if let Some(modal) = modals.last_mut() {
        modal.content.action(&mut ui.data, animations, action, handler);
        true
    } else {
        false
    }
}

/// Renders a modal window along with its contents
pub fn render_modal<T>(modal: &mut Modal<T>, data: &mut T, graphics: &mut crate::Graphics) {
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

    graphics.bounds = Rect::point_and_size(window_position, content_size);
    modal.content.render(graphics, data, &modal.content.get_id());
}

pub fn render_toasts(toasts: &Vec<Toast>, graphics: &mut crate::Graphics) {
    let count = toasts.len();
    let x = (graphics.bounds.right() + graphics.bounds.left()) / 2.;
    let y = graphics.bounds.bottom();

    const WIDTH: f32 = 500.;
    let font_size = graphics.font_size();
    let height = font_size * count as f32 + MARGIN * 2.;

    let rect = Rect::point_and_size(LogicalPosition::new(x - WIDTH / 2., y - height), LogicalSize::new(WIDTH, height));
    graphics.draw_rectangle(rect, MODAL_BACKGROUND);

    let mut curr_y = rect.top() + MARGIN;
    for toast in toasts {
        let text = crate::ui::get_drawable_text(graphics, graphics.font_size(), &toast.message);
        graphics.draw_text(LogicalPosition::new(x - text.width() / 2., curr_y), graphics.font_color(), &text);
        curr_y += font_size;
    }
}
