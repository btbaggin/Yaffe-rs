use crate::{YaffeState, Actions, Rect, LogicalPosition, LogicalSize, DeferredAction, windowing::WindowHelper};
use crate::assets::Images;
use crate::ui::controls::{MARGIN, MODAL_OVERLAY_COLOR, MODAL_BACKGROUND, change_brightness};


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

pub type ModalOnClose = fn(&mut YaffeState, ModalResult, &dyn ModalContent, &mut DeferredAction);
pub struct Modal {
    title: String,
    confirmation_button: Option<String>,
    content: Box<dyn ModalContent>,
    on_close: Option<ModalOnClose>,
}
impl Modal {
    pub fn overlay(content: Box<dyn ModalContent>) -> Modal {
        Modal { 
            title: String::from("Yaffe"), 
            confirmation_button: Some(String::from("Exit")),
            content, 
            on_close: None, 
        }
    }

    pub fn action(&mut self, action: &crate::Actions, helper: &mut crate::windowing::WindowHelper) -> ModalResult {
        self.content.action(action, helper)
    }
}

pub trait ModalContent {
    fn as_any(&self) -> &dyn std::any::Any;
    fn size(&self, rect: Rect, graphics: &crate::Graphics) -> LogicalSize;
    fn render(&self, rect: Rect, graphics: &mut crate::Graphics);
    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult;
    fn default_modal_action(action: &Actions) -> ModalResult where Self: Sized {
        match action {
            Actions::Accept => ModalResult::Ok,
            Actions::Back => ModalResult::Cancel,
            _ => ModalResult::None ,
        }
    }
    fn modal_width(rect: Rect, size: ModalSize) -> f32 where Self: Sized {
        match size {
            ModalSize::Third => rect.width() * 0.33,
            ModalSize::Half => rect.width() * 0.5,
            ModalSize::Full => rect.width(),
        }
    }
}

pub fn display_modal(state: &mut YaffeState, 
                     title: &str, 
                     confirmation_button: Option<&str>,
                     content: Box<dyn ModalContent>, 
                     on_close: Option<ModalOnClose>) {
    let confirm = confirmation_button.map(String::from);

    let m = Modal { 
        title: String::from(title), 
        confirmation_button: confirm,
        content, 
        on_close, 
    };
    
    let mut modals = state.modals.lock().unwrap();
    modals.push(m);
}

pub fn update_modal(state: &mut YaffeState, helper: &mut WindowHelper, action: &Actions, handler: &mut DeferredAction) {
    //This method can call into display_modal above, which locks the mutex
    //If we lock here that call will wait infinitely
    //We can get_mut here to ensure compile time exclusivity instead of locking
    //That allows us to call display_modal in close() below
    let modals = state.modals.get_mut().unwrap();
    if let Some(modal) = modals.last_mut() {
        let result = modal.content.action(action, helper);

        match result {
            ModalResult::Ok | ModalResult::Cancel => {
                let modal = modals.pop().unwrap();
                if let Some(close) = modal.on_close {
                    close(state, result, &*modal.content, handler);
                }
                
            }
            ModalResult::None => {},
        }
    }
}

pub fn is_modal_open(state: &YaffeState) -> bool {
    let modals = state.modals.lock().unwrap();
    modals.len() > 0
}

/// Renders a modal window along with its contents
pub fn render_modal(modal: &Modal, graphics: &mut crate::Graphics) {
    const TOOLBAR_SIZE: f32 = 18.;
    const TITLEBAR_SIZE: f32 = 32.;
    const PADDING: f32 = 2.;

    let padding = LogicalSize::new(graphics.bounds.width() * 0.1, graphics.bounds.height() * 0.1);
    let rect = Rect::new(*graphics.bounds.top_left() + padding, graphics.bounds.size() - padding);
    let content_size = modal.content.size(rect, graphics);

    //Calulate size
    let mut size = LogicalSize::new(MARGIN * 2. + content_size.x, MARGIN * 2. + TITLEBAR_SIZE + content_size.y);
    if modal.confirmation_button.is_some() {
        size.y += TOOLBAR_SIZE;
    }

    let window_position = (rect.size() - size) / 2. + padding;
    let window = Rect::new(window_position, window_position + size);
    
    //Background
    graphics.draw_rectangle(graphics.bounds, MODAL_OVERLAY_COLOR);
    graphics.draw_rectangle(window, MODAL_BACKGROUND);

    //Titlebar
    let titlebar_color = graphics.accent_color();
    let titlebar_color = change_brightness(&titlebar_color, graphics.light_shade_factor());
    let titlebar_pos = window_position + LogicalSize::new(PADDING, PADDING);
    let titlebar = Rect::new(titlebar_pos, titlebar_pos + LogicalSize::new(size.x - 4., TITLEBAR_SIZE));
    graphics.draw_rectangle(titlebar, titlebar_color);

    let title_text = crate::ui::get_drawable_text(graphics.font_size(), &modal.title);
    let title_pos = LogicalPosition::new(titlebar_pos.x + MARGIN, titlebar_pos.y + (TITLEBAR_SIZE - title_text.height()) / 2.);
    graphics.draw_text(title_pos, graphics.font_color(), &title_text);

    //Content
    //Window + margin for window + margin for icon
    let content_pos = LogicalPosition::new(window_position.x + MARGIN + PADDING, window_position.y + MARGIN + TITLEBAR_SIZE + PADDING); 
    let content_rect = Rect::new(content_pos, content_pos + content_size);
    modal.content.render(content_rect, graphics);

    //Action buttons
    if let Some(s) = &modal.confirmation_button {
        let mut right = LogicalPosition::new(window.right() - 5., window.bottom() - (TOOLBAR_SIZE + MARGIN));

        for t in [("Cancel", Images::ButtonB), (&s[..], Images::ButtonA)] {
            let text = crate::ui::get_drawable_text(TOOLBAR_SIZE, t.0);
            right = crate::ui::right_aligned_text(graphics, right, Some(t.1), graphics.font_color(), text);
            right.x -= MARGIN;
        }
    }
}