use druid_shell::kurbo::{Size, Rect, Point, RoundedRect};
use druid_shell::piet::{Piet, RenderContext};
use crate::{YaffeState, Actions};
use crate::settings::SettingNames;
use crate::colors::*;
use crate::assets::{request_preloaded_image, Images};

#[repr(u8)]
pub enum ModalResult {
    None,
    Ok,
    Cancel,
}

#[repr(u8)]
#[derive(Copy, Clone)]
pub enum ModalSize {
    Third,
    Half,
    Full,
}

type ModalOnClose = fn(&mut YaffeState, ModalResult, &Box<dyn ModalContent>);
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
    fn get_height(&self) -> f64;
    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, piet: &mut Piet);
    fn action(&mut self, action: &Actions, _: &mut DeferredModalAction) -> ModalResult { 
        default_modal_action(action)
    }
}

#[repr(u8)]
enum ModalFileAction {
    OpenFile,
    OpenDirectory,
}
pub struct DeferredModalAction {
    file_action: Option<ModalFileAction>,
}
impl DeferredModalAction {
    pub fn new() -> DeferredModalAction {
        DeferredModalAction { file_action: None }
    }
    pub fn open_file(&mut self) {
        self.file_action = Some(ModalFileAction::OpenFile);
    }
    pub fn open_directory(&mut self) {
        self.file_action = Some(ModalFileAction::OpenDirectory);
    }

    pub fn resolve(self, state: &mut YaffeState) {
        match self.file_action {
            Some(ModalFileAction::OpenFile) => state.win.handle.open_file(druid_shell::FileDialogOptions::new()),
            Some(ModalFileAction::OpenDirectory) => {
                let options = druid_shell::FileDialogOptions::new();
                let options = options.select_directories();
                state.win.handle.open_file(options)
            }
            None => None,
        };
    }
}

//Modal for displaying a simple string
pub struct MessageModalContent {
    message: String,
}
impl MessageModalContent {
    pub fn new(message: &str) -> MessageModalContent {
        MessageModalContent { message: String::from(message), }
    }
}
impl ModalContent for MessageModalContent {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_height(&self) -> f64 { crate::font::FONT_SIZE }
    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, piet: &mut Piet) {
        let name_label = crate::widgets::get_drawable_text(piet, crate::font::FONT_SIZE, &self.message, get_font_color(settings));
        piet.draw_text(&name_label, Point::new(rect.x0, rect.y0));
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

pub(crate) fn update_modal(state: &mut YaffeState, action: &Actions) {
    //This method can call into display_modal above, which locks the mutex
    //If we lock here that call will wait infinitely
    //We can get_mut here to ensure compile time exclusivity instead of locking
    //That allows us to call display_modal in close() below
    let mut handler = DeferredModalAction::new();
    let modals = state.modals.get_mut().unwrap();
    if let Some(modal) = modals.last_mut() {
        let result = modal.content.action(&action, &mut handler);

        match result {
            ModalResult::Ok | ModalResult::Cancel => {
                let modal = modals.pop().unwrap();
                if let Some(close) = modal.on_close {
                    close(state, result, &modal.content);
                }
                
            }
            ModalResult::None => {},
        }
    }
    handler.resolve(state);
}

pub(crate) fn is_modal_open(state: &YaffeState) -> bool {
    let modals = state.modals.lock().unwrap();
    modals.len() > 0
}

pub fn render_modal(settings: &crate::settings::SettingsFile, modal: &Modal, rect: &Rect, piet: &mut Piet) {
    const BUTTON_SIZE: f64 = 18.;
    const MARGIN: f64 = 10.;
    const TITLEBAR_SIZE: f64 = 32.;
    const ICON_SIZE: f64 = 32.;
    const ICON_SIZE_WITH_MARGIN: f64 = ICON_SIZE + MARGIN * 2.;

    let content_size = match modal.size {
        ModalSize::Third => Size::new(rect.width() * 0.33, modal.content.get_height()),
        ModalSize::Half => Size::new(rect.width() * 0.5, modal.content.get_height()),
        ModalSize::Full => Size::new(rect.width(), modal.content.get_height()),
    };

    //Calulate size
    let mut size = Size::new(MARGIN * 2. + content_size.width, MARGIN * 2. + TITLEBAR_SIZE + content_size.height);
    if let Some(_) = modal.icon {
        size.height = f64::max(ICON_SIZE_WITH_MARGIN, size.height);
        size.width += ICON_SIZE_WITH_MARGIN;
    }
    if let Some(_) = modal.confirmation_button {
        size.height += BUTTON_SIZE;
    }

    let window_position = Point::new((rect.width() - size.width) / 2., (rect.height() - size.height) / 2.);

    let window = Rect::from((window_position, size));
    
    //Background
    piet.fill(rect, &MODAL_OVERLAY_COLOR);
    piet.fill(RoundedRect::from_rect(window, 5.), &MODAL_BACKGROUND);

    //Titlebar
    let titlebar_color = get_accent_color(settings);
    let titlebar_color = change_brightness(&titlebar_color, settings.get_f64(SettingNames::LightShadeFactor));
    let titlebar_pos = Point::new(window_position.x + 2., window_position.y + 2.);
    let titlebar = Rect::from((titlebar_pos, Size::new(size.width - 4., TITLEBAR_SIZE)));
    piet.fill(RoundedRect::from_rect(titlebar, 5.),  &titlebar_color);

    let title_text = crate::widgets::get_drawable_text(piet, crate::font::FONT_SIZE, &modal.title, get_font_color(settings));
    piet.draw_text(&title_text, Point::new(titlebar_pos.x + crate::ui::MARGIN, titlebar_pos.y));

    //Icon
    let mut icon_position = Point::new(window_position.x + MARGIN, window_position.y + MARGIN + TITLEBAR_SIZE); //Window + margin for window + margin for icon
    if let Some(image) = modal.icon {
        let icon = request_preloaded_image(piet, image);
        let icon_rect = Rect::from((icon_position, Size::new(ICON_SIZE, ICON_SIZE)));
        icon.render(piet, icon_rect);
        icon_position.x += ICON_SIZE;
    }

    //Content
    let content_rect = Rect::from((Point::new(icon_position.x + 2., icon_position.y + 2.), content_size));
    modal.content.render(settings, content_rect, piet);

    //Action buttons
    if let Some(s) = &modal.confirmation_button {
        let text = crate::widgets::get_drawable_text(piet, BUTTON_SIZE, &s[..], get_font_color(settings));
        crate::widgets::right_aligned_text(piet, Point::new(window.x1 - 5., window.y1 - (BUTTON_SIZE + 10.)), Some(Images::ButtonA), text);
    }
}
