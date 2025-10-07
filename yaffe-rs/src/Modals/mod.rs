use crate::assets::Images;
use crate::controls::{MODAL_BACKGROUND, MODAL_OVERLAY_COLOR};
use crate::ui::{
    change_brightness, AnimationManager, ContainerSize, Justification, LayoutElement, RightAlignment, UiContainer,
    UiElement, WidgetId, MARGIN,
};
use crate::{Actions, Graphics, LogicalPosition, LogicalSize, Rect, YaffeState};
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};

mod info_modal;
mod list_modal;
mod message_modal;
mod platform_detail_modal;
mod restricted_modal;
mod scraper_modal;
mod settings_modal;

pub use info_modal::InfoModal;
pub use list_modal::ListModal;
pub use message_modal::MessageModal;
pub use platform_detail_modal::{on_add_platform_close, on_update_platform_close, PlatformDetailModal};
pub use restricted_modal::{on_restricted_modal_close, verify_restricted_action, RestrictedMode, SetRestrictedModal};
pub use scraper_modal::{on_game_found_close, on_platform_found_close, ScraperModal};
pub use settings_modal::{on_settings_close, SettingsModal};

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

pub trait ModalContent {
    fn as_any(&self) -> &dyn std::any::Any;
    fn action(
        &mut self,
        _animations: &mut AnimationManager,
        _action: &Actions,
        _handler: &mut ModalAction,
        _container: &mut UiContainer<(), ModalAction>,
    ) -> bool {
        false
    }
}

pub type ModalOnClose = fn(&mut YaffeState, bool, &ModalContentElement);
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
impl UiElement<(), ModalAction> for ModalTitlebar {
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

#[allow(unused_variables)]
pub struct ModalContentElement {
    position: LogicalPosition,
    size: LogicalSize,
    id: WidgetId,
    focus_group: bool,
    focus: Option<WidgetId>,
    content: Box<dyn ModalContent>,
    container: UiContainer<(), ModalAction>,
}
impl LayoutElement for ModalContentElement {
    fn layout(&self) -> Rect { Rect::point_and_size(self.position, self.size) }
    fn set_layout(&mut self, layout: Rect) {
        self.position = *layout.top_left();
        self.size = layout.size();
    }
    fn get_id(&self) -> WidgetId { self.id }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
impl ModalContentElement {
    pub fn new(content: impl ModalContent + 'static, focus_group: bool) -> ModalContentElement {
        ModalContentElement {
            position: LogicalPosition::new(0., 0.),
            size: LogicalSize::new(0., 0.),
            id: WidgetId::random(),
            focus_group,
            focus: None,
            content: Box::new(content),
            container: UiContainer::column(),
        }
    }
    pub fn get_content<T: 'static>(&self) -> &T { crate::convert_to!(&self.content, T) }
}
impl Deref for ModalContentElement {
    type Target = UiContainer<(), ModalAction>;
    fn deref(&self) -> &Self::Target { &self.container }
}
impl DerefMut for ModalContentElement {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.container }
}

impl UiElement<(), ModalAction> for ModalContentElement {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize { self.container.calc_size(graphics) }

    fn render(&mut self, graphics: &mut Graphics, state: &(), current_focus: &WidgetId) {
        let rect = self.layout();
        graphics.bounds = Rect::point_and_size(
            LogicalPosition::new(rect.left() + MARGIN, rect.top()),
            LogicalSize::new(rect.width() - (MARGIN * 2.), rect.height()),
        );
        self.container.render(graphics, state, &self.focus.unwrap_or(*current_focus));
    }

    fn action(
        &mut self,
        state: &mut (),
        animations: &mut AnimationManager,
        action: &Actions,
        handler: &mut ModalAction,
    ) -> bool {
        // See if we should close
        if handler.close_if_accept(action) {
            return true;
        }
        // If we have a focus group, move focus first
        if self.focus_group {
            match action {
                Actions::Up => {
                    self.focus = self.container.move_focus(self.focus, false);
                    return true;
                }
                Actions::Down => {
                    self.focus = self.container.move_focus(self.focus, true);
                    return true;
                }
                _ => {}
            }
        }
        // If current control is focused, handle that
        if let Some(focus) = self.focus {
            if let Some(widget) = self.container.find_widget_mut(focus) {
                return widget.action(state, animations, action, handler);
            }
        }

        // Otherwise custom handling
        if self.content.action(animations, action, handler, &mut self.container) {
            return true;
        }
        self.container.action(state, animations, action, handler)
    }
}

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
    content: ModalContentElement,
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
    content: ModalContentElement,
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
                let content = crate::convert_to!(modal.content.get_child(1).as_ref(), ModalContentElement);
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
