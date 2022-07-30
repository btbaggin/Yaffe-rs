mod list_modal;
mod overlay_modal;
mod restricted_modal;
mod platform_detail_modal;
mod settings_modal;

use crate::{YaffeState, Actions, Rect, LogicalPosition, LogicalSize, DeferredAction, windowing::WindowHelper};
use crate::settings::SettingNames;
use crate::colors::*;
use crate::assets::{request_image, Images};

pub use list_modal::{ListItem, ListModal};
pub use overlay_modal::OverlayModal;
pub use restricted_modal::{SetRestrictedModal, VerifyRestrictedModal};
pub use settings_modal::{SettingsModal, on_settings_close};
pub use platform_detail_modal::{PlatformDetailModal, on_add_platform_close, on_platform_found_close, on_game_found_close, on_update_platform_close};

#[repr(u8)]
pub enum ModalResult {
    None,
    Ok,
    Cancel,
}

#[allow(dead_code)]
#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ModalSize {
    Third,
    Half,
    Full,
}

type ModalOnClose = fn(&mut YaffeState, ModalResult, &Box<dyn ModalContent>, &mut DeferredAction);
pub struct Modal {
    title: String,
    confirmation_button: Option<String>,
    pub content: Box<dyn ModalContent>,
    on_close: Option<ModalOnClose>,
    icon: Option<Images>,
    size: ModalSize,
}
impl Modal {
    pub fn overlay(content: Box<dyn ModalContent>) -> Modal {
        Modal { 
            title: String::from("Yaffe"), 
            confirmation_button: None,
            content: content, 
            on_close: None, 
            icon: None,
            size: ModalSize::Third }
    }
}

pub trait ModalContent {
    fn as_any(&self) -> &dyn std::any::Any;
    fn get_height(&self, settings: &crate::settings::SettingsFile, graphics: &crate::Graphics, width: f32) -> f32;
    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &mut crate::Graphics);
    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult { 
        default_modal_action(action)
    }
}

//Modal for displaying a simple string
pub struct MessageModalContent {
    message: String,
}
impl MessageModalContent {
    pub fn new(message: &str) -> MessageModalContent {
        MessageModalContent {
            message: String::from(message), 
        }
    }
}
impl ModalContent for MessageModalContent {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_height(&self, settings: &crate::settings::SettingsFile, graphics: &crate::Graphics, width: f32) -> f32 { 
        let name_label = crate::widgets::get_drawable_text_with_wrap(crate::font::get_font_size(settings, graphics), &self.message, width);
        name_label.height()
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, graphics: &mut crate::Graphics) {
        let name_label = crate::widgets::get_drawable_text_with_wrap(crate::font::get_font_size(settings, graphics), &self.message, rect.width() * graphics.scale_factor);
        graphics.draw_text(*rect.top_left(), get_font_color(settings), &name_label);
    }
}

pub fn default_modal_action(action: &Actions) -> ModalResult {
    match action {
        Actions::Accept => ModalResult::Ok,
        Actions::Back => ModalResult::Cancel,
        _ => ModalResult::None ,
    }
}

pub fn display_modal(state: &mut YaffeState, 
                     title: &str, 
                     confirmation_button: Option<&str>,
                     content: Box<dyn ModalContent>, 
                     size: ModalSize, 
                     on_close: Option<ModalOnClose>) {
    display_modal_with_icon(state, title, confirmation_button, content, size, None, on_close);
}

pub fn display_modal_with_icon(state: &mut YaffeState, 
                               title: &str, 
                               confirmation_button: Option<&str>,
                               content: Box<dyn ModalContent>, 
                               size: ModalSize, 
                               icon: Option<Images>, 
                               on_close: Option<ModalOnClose>) {

    let confirm = if let Some(s) = confirmation_button { Some(String::from(s)) } else { None };

    let m = Modal { 
        title: String::from(title), 
        confirmation_button: confirm,
        content: content, 
        on_close: on_close, 
        icon: icon,
        size: size 
    };
    
    let mut modals = state.modals.lock().unwrap();
    modals.push(m);
}

pub(crate) fn update_modal(state: &mut YaffeState, helper: &mut WindowHelper, action: &Actions, handler: &mut DeferredAction) {
    //This method can call into display_modal above, which locks the mutex
    //If we lock here that call will wait infinitely
    //We can get_mut here to ensure compile time exclusivity instead of locking
    //That allows us to call display_modal in close() below
    let modals = state.modals.get_mut().unwrap();
    if let Some(modal) = modals.last_mut() {
        let result = modal.content.action(&action, helper);

        match result {
            ModalResult::Ok | ModalResult::Cancel => {
                let modal = modals.pop().unwrap();
                if let Some(close) = modal.on_close {
                    close(state, result, &modal.content, handler);
                }
                
            }
            ModalResult::None => {},
        }
    }
}

pub(crate) fn is_modal_open(state: &YaffeState) -> bool {
    let modals = state.modals.lock().unwrap();
    modals.len() > 0
}

/// Renders a modal window along with its contents
pub fn render_modal(settings: &crate::settings::SettingsFile, modal: &Modal, graphics: &mut crate::Graphics) {
    const BUTTON_SIZE: f32 = 18.;
    const MARGIN: f32 = 10.;
    const TITLEBAR_SIZE: f32 = 32.;
    const ICON_SIZE: f32 = 32.;
    const ICON_SIZE_WITH_MARGIN: f32 = ICON_SIZE + MARGIN * 2.;

    let rect = graphics.bounds;
    let content_size = match modal.size {
        ModalSize::Third => {
            let width = rect.width() * 0.33;
            LogicalSize::new(width, modal.content.get_height(settings, graphics, width))
        }
        ModalSize::Half => {
            let width = rect.width() * 0.5;
            LogicalSize::new(width, modal.content.get_height(settings, graphics, width))
        }
        ModalSize::Full => {
            let width = rect.width();
            LogicalSize::new(width, modal.content.get_height(settings, graphics, width))
        }
    };

    //Calulate size
    let mut size = LogicalSize::new(MARGIN * 2. + content_size.x, MARGIN * 2. + TITLEBAR_SIZE + content_size.y);
    if let Some(_) = modal.icon {
        size.y = f32::max(ICON_SIZE_WITH_MARGIN, size.y);
        size.x += ICON_SIZE_WITH_MARGIN;
    }
    if let Some(_) = modal.confirmation_button {
        size.y += BUTTON_SIZE;
    }

    let window_position = (rect.size() - size) / 2.;
    let window = Rect::new(window_position, window_position + size);
    
    //Background
    graphics.draw_rectangle(rect, MODAL_OVERLAY_COLOR);
    graphics.draw_rectangle(window, MODAL_BACKGROUND);

    //Titlebar
    let titlebar_color = get_accent_color(settings);
    let titlebar_color = change_brightness(&titlebar_color, settings.get_f32(SettingNames::LightShadeFactor));
    let titlebar_pos = window_position + LogicalSize::new(2., 2.);
    let titlebar = Rect::new(titlebar_pos, titlebar_pos + LogicalSize::new(size.x - 4., TITLEBAR_SIZE));
    graphics.draw_rectangle(titlebar, titlebar_color);

    let title_text = crate::widgets::get_drawable_text(crate::font::get_font_size(settings, graphics), &modal.title);
    graphics.draw_text(LogicalPosition::new(titlebar_pos.x + crate::ui::MARGIN, titlebar_pos.y), get_font_color(settings), &title_text);

    //Icon
    let mut icon_position = LogicalPosition::new(window_position.x + MARGIN, window_position.y + MARGIN + TITLEBAR_SIZE); //Window + margin for window + margin for icon
    if let Some(image) = modal.icon {
        let icon = request_image(graphics, image).unwrap();
        let icon_rect = Rect::new(icon_position, icon_position + LogicalSize::new(ICON_SIZE, ICON_SIZE));

        icon.render(graphics, icon_rect);
        icon_position.x += ICON_SIZE;
    }

    //Content
    let content_pos = icon_position + LogicalSize::new(2., 2.);
    let content_rect = Rect::new(content_pos, content_pos + content_size);
    modal.content.render(settings, content_rect, graphics);

    //Action buttons
    if let Some(s) = &modal.confirmation_button {
        let text = crate::widgets::get_drawable_text(BUTTON_SIZE, &s[..]);
        crate::widgets::right_aligned_text(graphics, LogicalPosition::new(window.right() - 5., window.bottom() - (BUTTON_SIZE + 10.)), Some(Images::ButtonA), get_font_color(settings), text);
    }
}

pub fn outline_rectangle(graphics: &mut crate::Graphics, rect: &Rect, size: f32, color: speedy2d::color::Color) {
    let top_left = *rect.top_left();
    let bottom_right = *rect.bottom_right();
    let top_right = LogicalPosition::new(bottom_right.x, top_left.y);
    let bottom_left = LogicalPosition::new(top_left.x, bottom_right.y);

    graphics.draw_line(top_left, top_right, size, color);
    graphics.draw_line(top_right, bottom_right, size, color);
    graphics.draw_line(bottom_right, bottom_left, size, color);
    graphics.draw_line(bottom_left, top_left, size, color);
}