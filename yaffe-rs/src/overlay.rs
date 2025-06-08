use crate::logger::LogEntry;
use crate::state::ExternalProcess;
use crate::ui::{AnimationManager, Modal, ModalResult, WidgetContainer};
use crate::windowing::WindowHelper;
use crate::Graphics;
use std::cell::RefCell;
use std::rc::Rc;

/// Contains information needed to process and render
/// the Yaffe game overlay
pub struct OverlayWindow {
    modal: Modal,
    process: Option<Box<dyn ExternalProcess>>,
    showing: bool,
    settings: crate::settings::SettingsFile,
    root: WidgetContainer,
}
impl OverlayWindow {
    /// Returns a default `OverlayWindow` instance
    pub fn new(root: WidgetContainer, settings: crate::settings::SettingsFile) -> Rc<RefCell<OverlayWindow>> {
        let overlay = OverlayWindow {
            modal: Modal::overlay(Box::new(crate::modals::OverlayModal::new())),
            process: None,
            showing: false,
            settings,
            root,
        };

        Rc::new(RefCell::new(overlay))
    }

    pub fn is_active(&self) -> bool { self.process.is_some() }

    /// Sets the currently running process
    pub fn set_process(&mut self, process: Box<dyn ExternalProcess>) { self.process = Some(process); }

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
    fn on_frame_begin(&mut self, graphics: &mut Graphics, _: &mut Vec<crate::job_system::JobResult>) {
        crate::assets::preload_assets(graphics);
    }

    fn on_frame(&mut self, graphics: &mut Graphics) -> bool {
        graphics.cache_settings(&self.settings);
        crate::ui::render_modal(&self.modal, graphics);
        true
    }

    fn on_fixed_update(&mut self, helper: &mut WindowHelper) -> bool { !self.process_is_running(helper) }

    fn on_input(&mut self, _: &mut AnimationManager, helper: &mut WindowHelper, action: &crate::Actions) -> bool {
        if self.process.is_none() {
            return false;
        }
        match action {
            crate::Actions::ToggleOverlay => {
                self.toggle_visibility(helper);
                true
            }
            _ => {
                if self.showing {
                    let result = self.modal.action(action, helper);
                    if let ModalResult::Ok = result {
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

    fn get_ui(&mut self) -> &mut crate::ui::WidgetContainer { &mut self.root }
}
