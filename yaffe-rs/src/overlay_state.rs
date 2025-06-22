use crate::assets::AssetKey;
use crate::logger::LogEntry;
use crate::ui::WindowState;
use crate::windowing::WindowHelper;
use std::cell::RefCell;
use std::ops::{Deref, DerefMut};
use std::rc::Rc;

pub trait ExternalProcess {
    fn is_running(&mut self) -> bool;
    fn kill(&mut self) -> std::io::Result<()>;
}
impl ExternalProcess for std::process::Child {
    fn is_running(&mut self) -> bool {
        match self.try_wait() {
            Ok(None) => true,
            Ok(Some(_)) => false,
            Err(_) => {
                //If we cant kill it, oh well.
                self.kill().log("Unable to determine process status");
                false
            }
        }
    }
    fn kill(&mut self) -> std::io::Result<()> { self.kill() }
}

pub struct YaffeProcess {
    pub name: String,
    pub image: AssetKey,
    process: Box<dyn ExternalProcess>,
}
impl YaffeProcess {
    pub fn new(name: &str, image: AssetKey, process: Box<dyn ExternalProcess>) -> YaffeProcess {
        YaffeProcess { name: name.to_string(), image, process }
    }
}
impl Deref for YaffeProcess {
    type Target = Box<dyn ExternalProcess>;
    fn deref(&self) -> &Box<dyn ExternalProcess> { &self.process }
}
impl DerefMut for YaffeProcess {
    fn deref_mut(&mut self) -> &mut Box<dyn ExternalProcess> { &mut self.process }
}

/// Contains information needed to process and render
/// the Yaffe game overlay
pub struct OverlayState {
    pub process: Rc<RefCell<Option<YaffeProcess>>>,
    pub showing: bool,
    pub settings: crate::settings::SettingsFile,
}
impl OverlayState {
    /// Returns a default `OverlayWindow` instance
    pub fn new(process: Rc<RefCell<Option<YaffeProcess>>>, settings: crate::settings::SettingsFile) -> OverlayState {
        OverlayState { process, showing: false, settings }
    }

    /// Checks if a process is currently running
    /// If if has been killed in the background it will set
    /// process = None and hide the overlay
    pub fn process_is_running(&mut self, helper: &mut WindowHelper) -> bool {
        let mut process = self.process.borrow_mut();
        if process.is_some() {
            if !process.as_mut().unwrap().is_running() {
                *process = None;
                helper.set_visibility(false);
                self.showing = false;
                // self.hide(helper);
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
}
impl WindowState for OverlayState {
    fn on_revert_focus(&mut self) {}
}
