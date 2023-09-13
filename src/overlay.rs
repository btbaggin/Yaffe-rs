use crate::{Rect, PhysicalSize, LogicalPosition};
use crate::logger::LogEntry;
use crate::ui;
use crate::windowing::WindowHelper;
use std::rc::Rc;
use std::cell::RefCell;

/// Contains information needed to process and render
/// the Yaffe game overlay
pub struct OverlayWindow {
    modal: ui::Modal,
    process: Option<std::process::Child>,
    showing: bool,
    settings: crate::settings::SettingsFile,
}
impl OverlayWindow {
    /// Returns a default `OverlayWindow` instance
    pub fn new(settings: crate::settings::SettingsFile) -> Rc<RefCell<OverlayWindow>> {
        let overlay = OverlayWindow {
            modal: ui::Modal::overlay(Box::new(crate::modals::OverlayModal::new())),
            process: None,
            showing: false,
            settings,
        };
  
        Rc::new(RefCell::new(overlay))
    }

    pub fn is_active(&self) -> bool {
        self.process.is_some()
    }

    /// Sets the currently running process
    pub fn set_process(&mut self, process: std::process::Child) {
        self.process = Some(process);
    }

    /// Checks if a process is currently running
    /// If if has been killed in the background it will set
    /// process = None and hide the overlay
    pub fn process_is_running(&mut self, helper: &mut WindowHelper) -> bool {
        if let Some(process) = &mut self.process {
            match process.try_wait() { 
                Ok(None) => true,
                Ok(Some(_)) => {
                    self.process = None;
                    self.hide(helper);
                    false
                },
                Err(_) => {
                    //If we cant kill it, oh well.
                    process.kill().log("Unable to determine process status");
                    self.hide(helper);
                    false
                }
            }
        } else { 
            false 
        }
    }

    /// Shows the overlay if possible
    pub fn toggle_visibility(&mut self, helper: &mut WindowHelper) {
        self.showing = !self.showing;
        helper.set_visibility(self.showing);
    }

    fn hide(&mut self, helper: &mut WindowHelper) {
        helper.set_visibility(false);
        self.showing = false;
    }
}

impl crate::windowing::WindowHandler for OverlayWindow {
    fn on_frame(&mut self, graphics: &mut speedy2d::Graphics2D, _: f32, size: PhysicalSize, scale_factor: f32) -> bool {
        let window_rect = Rect::new(LogicalPosition::new(0., 0.), size.to_logical(scale_factor));

            let mut graphics = crate::Graphics { graphics, queue: None, scale_factor, bounds: window_rect, delta_time: 0. };
            crate::ui::render_modal(&self.settings, &self.modal, &mut graphics);

        true
    }

    fn on_fixed_update(&mut self, helper: &mut WindowHelper, _: &mut Vec<crate::job_system::JobResult>) -> bool {
        !self.process_is_running(helper)
    }

    fn on_input(&mut self, helper: &mut WindowHelper, action: &crate::Actions) -> bool {
        if self.process.is_none() { return false; }
        match action {
            crate::Actions::ToggleOverlay => {
                self.toggle_visibility(helper);
                true
            }
            _ => {
                if self.showing { 
                    let result = self.modal.action(action, helper);
                    if let ui::ModalResult::Ok = result {
                        // It's safe to unwrap here because we are guaranteed to have a process or this window wouldn't be open
                        // see process_is_running
                        self.process.as_mut().unwrap().kill().log("Unable to kill running process");
                        self.process = None;
                        self.hide(helper);
                        return true;
                    }
                }
                false
            }
        }
    }

    fn is_window_dirty(&self) -> bool {
        self.showing
    }
}