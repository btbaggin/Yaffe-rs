use crate::controls::{MODAL_BACKGROUND, MODAL_OVERLAY_COLOR};
use crate::ui::{
    AnimationManager, ContainerSize, DeferredAction, Justification, LayoutElement, UiContainer, UiElement, WidgetTree,
    MARGIN,
};
use crate::{Actions, LogicalPosition, LogicalSize, Rect};
use std::ops::{Deref, DerefMut};

mod info_modal;
mod menu_modal;
mod message_modal;
mod modal_content;
mod modal_deferred_actions;
mod platform_detail_modal;
mod restricted_modal;
mod scraper_modal;
mod settings_modal;

pub use info_modal::InfoModal;
pub use menu_modal::MenuModal;
pub use message_modal::MessageModal;
pub use modal_content::ModalContentElement;
pub use modal_deferred_actions::{DisplayModal, ModalClose};
pub use platform_detail_modal::PlatformDetailModal;
pub use restricted_modal::{verify_restricted_action, RestrictedMode, SetRestrictedModal};
pub use scraper_modal::ScraperModal;
pub use settings_modal::SettingsModal;

use modal_content::{ModalTitlebar, ModalToolbar};

pub struct Toast {
    message: String,
    time: f32,
}
impl Toast {
    pub fn new(message: &str, time: f32) -> Toast { Toast { message: message.to_string(), time } }

    pub fn process_toast(toasts: &mut Vec<Toast>, delta_time: f32) {
        toasts.retain(|t| t.time > 0.);
        for t in toasts.iter_mut() {
            t.time -= delta_time;
        }
    }
}

#[allow(dead_code)]
#[derive(Copy, Clone)]
pub enum ModalSize {
    Third,
    Half,
    Full,
}

pub enum ModalValidationResult {
    Ok,
    Cancel(String),
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
    fn validate(&self, _content: &UiContainer<T>) -> ModalValidationResult { ModalValidationResult::Ok }
    fn on_close(&self, state: &mut T, accept: bool, container: &UiContainer<T>, handler: &mut DeferredAction<T>);
}

pub struct Modal<T: 'static> {
    content: Box<UiContainer<T>>,
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
    display_modal_raw(ui, "Error", None, message, ModalSize::Half);
}

pub fn display_modal_raw<T>(
    ui: &mut WidgetTree<T>,
    title: &str,
    confirmation_button: Option<&str>,
    content: ModalContentElement<T>,
    width: ModalSize,
) {
    let confirm = confirmation_button.map(String::from);

    let content = build_modal(String::from(title), confirm, content);
    let m = Modal { content: Box::new(content), width };

    let mut modals = ui.modals.lock().unwrap();
    modals.push(m);
}

pub fn update_modal<T>(ui: &mut WidgetTree<T>, action: &Actions, handler: &mut DeferredAction<T>) -> bool {
    let modals = ui.modals.get_mut().unwrap();
    if let Some(modal) = modals.last_mut() {
        modal.content.action(&mut ui.data, &mut ui.animations, action, handler);
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

pub fn render_toasts(toasts: &[Toast], graphics: &mut crate::Graphics) {
    const MAX_WIDTH: f32 = 500.;

    let message = toasts.iter().map(|t| t.message.as_str()).collect::<Vec<_>>().join("\n");
    let text = crate::ui::get_drawable_text(graphics, graphics.font_size(), &message);
    let height = text.height() + MARGIN * 2.;
    let width = f32::max(text.width() + MARGIN * 2., MAX_WIDTH);

    let x = (graphics.bounds.right() + graphics.bounds.left()) / 2.;
    let y = graphics.bounds.bottom();

    let rect = Rect::point_and_size(LogicalPosition::new(x - width / 2., y - height), LogicalSize::new(width, height));
    graphics.draw_rectangle(rect, MODAL_BACKGROUND);
    graphics.draw_text(LogicalPosition::new(x - text.width() / 2., rect.top() + MARGIN), graphics.font_color(), &text);
}
