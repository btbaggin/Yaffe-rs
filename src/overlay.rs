use speedy2d::shape::Rectangle;
use speedy2d::dimen::Vector2;
use crate::{modals, platform_layer::ControllerInput};
use glutin::event::VirtualKeyCode;

/// Contains information needed to process and render
/// the Yaffe game overlay
pub struct OverlayWindow {
    modal: modals::Modal,
    process: Option<std::process::Child>,
    showing: bool,
    settings: crate::settings::SettingsFile,
}
impl OverlayWindow {
    /// Returns a default `OverlayWindow` instance
    pub fn new(settings: crate::settings::SettingsFile) -> std::rc::Rc<std::cell::RefCell<OverlayWindow>> {
        let overlay = OverlayWindow {
            modal: modals::Modal::overlay(Box::new(modals::OverlayModal::default())),
            process: None,
            showing: false,
            settings: settings,
        };
  
        std::rc::Rc::new(std::cell::RefCell::new(overlay))
    }

    pub fn is_showing(&self) -> bool {
        self.showing
    }

    /// Sets the currently running process
    pub fn set_process(&mut self, process: std::process::Child) {
        self.process = Some(process);
    }

    /// Checks if a process is currently running
    /// If if has been killed in the background it will set
    /// process = None and hide the overlay
    pub fn process_is_running(&mut self) -> bool {
        if let Some(process) = &mut self.process {
            match process.try_wait() { 
                Ok(None) => true,
                Ok(Some(_)) => {
                    self.process = None;
                    // self.hide();
                    false
                },
                Err(_) => {
                    //If we cant kill it, oh well.
                    if let Err(e) = process.kill() {
                        crate::logger::log_entry_with_message(crate::logger::LogTypes::Warning, e, "Unable to determine process status");
                    }
                    // self.hide();
                    false
                }
            }
        } else { 
            false 
        }
    }

    /// Shows the overlay if possible
    pub fn toggle_visibility(&mut self, helper: &mut crate::windowing::WindowHelper) {
        self.showing = !self.showing;
        helper.set_visibility(self.showing);
    }

    fn hide(&mut self, helper: &mut crate::windowing::WindowHelper) {
        helper.set_visibility(false);
        self.showing = false;
    }
}

impl crate::windowing::WindowHandler for OverlayWindow {
    fn on_frame(&mut self, graphics: &mut speedy2d::Graphics2D, size: Vector2<u32>) -> bool {
        let window_rect = Rectangle::from_tuples((0., 0.), (size.x as f32, size.y as f32));
        modals::render_modal(&self.settings, &self.modal, &window_rect, graphics);

        true
    }

    fn on_input(&mut self, helper: &mut crate::windowing::WindowHelper, key: Option<VirtualKeyCode>, button: Option<ControllerInput>) -> bool {
        if let None = self.process { return false; }
        if let Some(crate::SystemActions::ToggleOverlay) = crate::SYSTEM_ACTION_MAP.get(key, button) { 
            self.toggle_visibility(helper); 
            return true;
        };
    
        if self.showing { 
            let action = crate::ACTION_MAP.get(key, button);
            let mut handler = crate::modals::modal::DeferredModalAction::new();
            if let Some(a) = action {
                let result = self.modal.content.action(a, &mut handler);
                if let modals::ModalResult::Ok = result {
                    // It's safe to unwrap here because we are guaranteed to have a process or this window wouldn't be open
                    // see process_is_running
                    if let Err(e) = self.process.as_mut().unwrap().kill() {
                        crate::logger::log_entry_with_message(crate::logger::LogTypes::Warning, e, "Unable to kill running process");
                    }
                    self.process = None;
                    self.hide(helper);
                    return true;
                }
            }
        }

        false
    }
}