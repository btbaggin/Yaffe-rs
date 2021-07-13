use speedy2d::{Graphics2D, Window};
use speedy2d::shape::Rectangle;
use speedy2d::window::*;
use crate::{modals};

/// Contains information needed to process and render
/// the Yaffe game overlay
pub struct OverlayWindow {
    size: speedy2d::dimen::Vector2<u32>,
    modal: modals::Modal,
    process: Option<std::process::Child>,
    showing: bool,
    settings: crate::settings::SettingsFile,
}
impl OverlayWindow {
    /// Returns a default `OverlayWindow` instance
    pub fn new(settings: crate::settings::SettingsFile) -> std::rc::Rc<std::cell::RefCell<OverlayWindow>> {
        let overlay = OverlayWindow {
            size: speedy2d::dimen::Vector2::new(0, 0),
            modal: modals::Modal::overlay(Box::new(modals::OverlayModal::default())),
            process: None,
            showing: false,
            settings: settings,
        };

        // let event_loop = EventLoop::new();
        // let context_builder = glutin::ContextBuilder::new()
        // .with_gl_debug_flag(true)
        // .with_multisampling(0)
        // .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (2, 0)));

        // #[cfg(not(target_os = "linux"))]
        // let context = context_builder
        //     .build_windowed(
        //         glutin::window::WindowBuilder::new()
        //             .with_inner_size(PhysicalSize::new(width, height)),
        //         &event_loop
        //     )
        //     .unwrap();

        // #[cfg(target_os = "linux")]
        // let context = context_builder
        //     .with_vsync(false)
        //     .build_headless(&event_loop, PhysicalSize::new(width, height))
        //     .unwrap();

        // let context = unsafe { context.make_current().unwrap() };

        // // Used for glReadPixels/etc
        // gl::load_with(|ptr| context.get_proc_address(ptr) as *const _);

        // let mut renderer = unsafe {
        //     GLRenderer::new_for_gl_context((width, height), |name| {
        //         context.get_proc_address(name) as *const _
        //     })
        //     .unwrap()
        // };
        // let mut renderer = unsafe {
        //     GLRenderer::new_for_gl_context((640, 480), |fn_name| {
        //         window_context.get_proc_address(fn_name) as *const _
        //     })
        // }.unwrap();

        std::rc::Rc::new(std::cell::RefCell::new(overlay))
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
            //TODO self.handle.set_window_state(WindowState::Minimized);

        } else {
            if let Some(_) = self.process {
                //TODO self.handle.show();
                //TODO self.handle.bring_to_front_and_focus();
                self.showing = true;
            }
        }
        self.showing = !self.showing;
    }

    fn hide(&mut self) {
        //TODO self.handle.set_window_state(WindowState::Minimized);
        self.showing = false;
    }
}

impl WindowHandler for OverlayWindow {
    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
        let size = self.size;
        let window_rect = Rectangle::from_tuples((0., 0.), (size.x as f32, size.y as f32));

        modals::render_modal(&self.settings, &self.modal, &window_rect, graphics);
        helper.request_redraw();
    }

    fn on_key_down(&mut self, _: &mut WindowHelper, virtual_key_code: Option<VirtualKeyCode>, _: KeyScancode) {
        handle_input(self, virtual_key_code, None);
    }

    fn on_resize(&mut self, _: &mut WindowHelper, size_pixels: speedy2d::dimen::Vector2<u32>) { 
        self.size = size_pixels
    }
}

pub fn handle_input(tree: &mut OverlayWindow, code: Option<VirtualKeyCode>, button: Option<u16>) -> bool {
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
