use druid_shell::{
    Application, KeyEvent, Code,
    Region, WinHandler, WindowBuilder, WindowHandle, WindowState,
};
use druid_shell::kurbo::Size;
use druid_shell::piet::Piet;
use std::any::Any;
use crate::modals;

/// Contains information needed to process and render
/// the Yaffe game overlay
pub struct OverlayWindow {
    handle: WindowHandle,
    size: Size,
    modal: modals::Modal,
    process: Option<std::process::Child>,
    showing: bool,
    settings: crate::settings::SettingsFile,
}
impl OverlayWindow {
    /// Returns a default `OverlayWindow` instance
    pub fn new(settings: crate::settings::SettingsFile) -> *mut OverlayWindow {

        let window = OverlayWindow {
            handle: WindowHandle::default(),
            size: Size::default(),
            modal: modals::Modal::overlay(Box::new(modals::OverlayModal::default())),
            process: None,
            showing: false,
            settings: settings,
        };

        let mut overlay = WindowBuilder::new(Application::global().clone());
        let mut handler = Box::new(window);

        let handler_ptr = &mut *handler as *mut OverlayWindow;
        
        overlay.set_handler(handler);
        overlay.set_transparent(true);
        overlay.resizable(false);
        
        let mut handle = overlay.build().unwrap();
        handle.set_window_state(WindowState::Maximized); 
        handle.show_titlebar(false);

        handler_ptr
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
                    self.hide();
                    false
                },
                Err(_) => {
                    //If we cant kill it, oh well.
                    if let Err(e) = process.kill() {
                        crate::logger::log_entry_with_message(crate::logger::LogTypes::Warning, e, "Unable to determine process status");
                    }
                    self.hide();
                    false
                }
            }
        } else { 
            false 
        }
    }

    /// Shows the overlay if possible
    pub fn toggle_visibility(&mut self) {
        if self.showing {
            self.handle.set_window_state(WindowState::Minimized);

        } else {
            if let Some(_) = self.process {
                self.handle.show();
                self.handle.bring_to_front_and_focus();
                self.showing = true;
            }
        }
        self.showing = !self.showing;
    }

    fn hide(&mut self) {
        self.handle.set_window_state(WindowState::Minimized);
        self.showing = false;
    }
}

impl WinHandler for OverlayWindow {
    fn connect(&mut self, handle: &WindowHandle) { 
        self.handle = handle.clone(); 
    }
    fn prepare_paint(&mut self) { self.handle.invalidate(); }

    fn paint(&mut self, piet: &mut Piet, _: &Region) {
        let size = self.size;
        let window_rect = size.to_rect();

        //TODO read in settings for overlay
        modals::render_modal(&self.settings, &self.modal, &window_rect, piet);
        self.handle.request_anim_frame();
    }

    fn key_down(&mut self, event: KeyEvent) -> bool {
        handle_input(self, Some(event.code), None)
    }

    fn size(&mut self, size: Size) { self.size = size; }
    fn as_any(&mut self) -> &mut dyn Any { self }
}

pub fn handle_input(tree: &mut OverlayWindow, code: Option<Code>, button: Option<u16>) -> bool {
    if let Some(crate::SystemActions::ToggleOverlay) = crate::SYSTEM_ACTION_MAP.get(code, button) { 
        tree.toggle_visibility(); 
        return true;
    };

    let action = crate::ACTION_MAP.get(code, button);
    let mut handler = crate::modals::modal::DeferredModalAction::new();
    if let Some(a) = action {
        let result = tree.modal.content.action(a, &mut handler);
        if let modals::ModalResult::Ok = result {
            // It's safe to unwrap here because we are guaranteed to have a process or this window wouldn't be open
            // see process_is_running
            if let Err(e) = tree.process.as_mut().unwrap().kill() {
                crate::logger::log_entry_with_message(crate::logger::LogTypes::Warning, e, "Unable to kill running process");
            }
            tree.process = None;
            tree.hide();
            return true;
        }
    }

    false
}
