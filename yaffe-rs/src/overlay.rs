use crate::job_system::ThreadSafeJobQueue;
use crate::state::ExternalProcess;
use crate::{Rect, PhysicalSize, LogicalPosition, Graphics};
use crate::logger::LogEntry;
use crate::ui;
use crate::windowing::WindowHelper;
use std::rc::Rc;
use std::cell::RefCell;

/// Contains information needed to process and render
/// the Yaffe game overlay
pub struct OverlayWindow {
    modal: ui::Modal,
    process: Option<Box<dyn ExternalProcess>>,
    showing: bool,
    settings: crate::settings::SettingsFile,
    queue: ThreadSafeJobQueue
}
impl OverlayWindow {
    /// Returns a default `OverlayWindow` instance
    pub fn new(settings: crate::settings::SettingsFile, queue: ThreadSafeJobQueue) -> Rc<RefCell<OverlayWindow>> {
        let overlay = OverlayWindow {
            modal: ui::Modal::overlay(Box::new(crate::modals::OverlayModal::new())),
            process: None,
            showing: false,
            settings,
            queue,
        };
  
        Rc::new(RefCell::new(overlay))
    }

    pub fn is_active(&self) -> bool {
        self.process.is_some()
    }

    /// Sets the currently running process
    pub fn set_process(&mut self, process: Box<dyn ExternalProcess>) {
        self.process = Some(process);
    }

    /// Checks if a process is currently running
    /// If if has been killed in the background it will set
    /// process = None and hide the overlay
    pub fn process_is_running(&mut self, helper: &mut WindowHelper) -> bool {
        if let Some(process) = &mut self.process {
            if !process.is_running() {
                self.hide(helper);
                self.process = None;
                return false;
            }
            return true;
        }
        false
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
        let data = graphics.create_image_from_file_path(None, speedy2d::image::ImageSmoothingMode::Linear,"./Assets/packed.png").ok();

        let window_rect = Rect::new(LogicalPosition::new(0., 0.), size.to_logical(scale_factor));

        let mut graphics = Graphics::new(graphics, self.queue.clone(), scale_factor, window_rect, 0.);
        graphics.cache_settings(&self.settings);
        crate::ui::render_modal(&self.modal, &mut graphics);

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