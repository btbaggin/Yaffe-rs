use crate::{
    graphics::Graphics,
    input::ControllerInput,
    input::InputType,
    job_system::{Job, JobResult, JobResults, ThreadSafeJobQueue},
    logger::{LogEntry, PanicLogEntry},
    ui::AnimationManager,
    Actions, PhysicalSize,
};
use std::cell::RefCell;
use std::rc::Rc;

use glutin::surface::WindowSurface;

use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowId, WindowAttributes, WindowLevel};

use speedy2d::GLRenderer;

// mod context_tracker;
// use context_tracker::{ContextTracker, ContextWrapper};

mod app;

const UPDATE_TIMER: f32 = 60. * 60.;

#[repr(u8)]
enum WindowVisibility {
    Visible,
    Hide,
}

pub struct WindowHelper {
    visible: Option<WindowVisibility>,
}

impl WindowHelper {
    pub fn new() -> WindowHelper {
        WindowHelper { visible: None }
    }

    pub fn set_visibility(&mut self, visible: bool) {
        if visible {
            self.visible = Some(WindowVisibility::Visible);
        } else {
            self.visible = Some(WindowVisibility::Hide);
        }
    }

    pub fn resolve(self, window: &Window) {
        match self.visible {
            Some(WindowVisibility::Hide) => window.set_visible(false),
            Some(WindowVisibility::Visible) => window.set_visible(true),
            None => {}
        }
    }
}

pub(crate) trait WindowHandler {
    fn on_fixed_update(&mut self, helper: &mut WindowHelper) -> bool;
    fn on_frame_begin(&mut self, _: &mut Graphics, _: &mut Vec<JobResult>) {}
    fn on_frame(&mut self, graphics: &mut Graphics) -> bool;
    fn on_input(
        &mut self,
        animations: &mut AnimationManager,
        helper: &mut WindowHelper,
        action: &crate::Actions,
    ) -> bool;
    fn on_init(&mut self, graphics: &mut Graphics);
    fn on_stop(&mut self) {}
    fn get_ui(&mut self) -> &mut crate::ui::WidgetContainer;
}

struct YaffeWindow {
    window: Window,
    gl_context: glutin::context::PossiblyCurrentContext,
    gl_surface: glutin::surface::Surface<WindowSurface>,
    renderer: GLRenderer,
    size: PhysicalSize,
    handler: std::rc::Rc<RefCell<dyn WindowHandler + 'static>>,
    graphics: RefCell<Graphics>,
    animations: RefCell<AnimationManager>,
}

struct WindowInfo {
    handler: std::rc::Rc<RefCell<dyn WindowHandler + 'static>>,
    attributes: WindowAttributes,
    fullscreen: bool
}
impl WindowInfo {
    pub fn new(handler: std::rc::Rc<RefCell<dyn WindowHandler + 'static>>, attributes: WindowAttributes, fullscreen: bool) -> WindowInfo {
        WindowInfo { handler, attributes, fullscreen }
    }
}

pub(crate) fn create_yaffe_windows(
    job_results: JobResults,
    queue: ThreadSafeJobQueue,
    gamepad: impl crate::input::PlatformGamepad + 'static,
    input_map: crate::input::InputMap<KeyCode, ControllerInput, Actions>,
    handler: Rc<RefCell<dyn WindowHandler + 'static>>,
    overlay: Rc<RefCell<dyn WindowHandler + 'static>>,
) {
    let el = EventLoop::new().unwrap();
    el.set_control_flow(ControlFlow::Poll);

    let main = WindowAttributes::default().with_title("Yaffe").with_visible(true);

    let overlay_att = WindowAttributes::default()
        .with_title("Overlay")
        // .with_inner_size(PhysicalSize::new(size.x - 1., size.y - 1.))
        // .with_position(PhysicalPosition::new(1i32, 1i32))
        .with_visible(false)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_transparent(true)
        .with_decorations(false);

    let handlers = vec!(WindowInfo::new(handler, main, true), WindowInfo::new(overlay, overlay_att, true));

    let mut app = app::App::new(input_map, queue, handlers, job_results, gamepad);
    let _ = el.run_app(&mut app);
}

fn handle_action(window: &mut YaffeWindow, action: &crate::Actions) -> bool {
    let mut helper = WindowHelper::new();
    let mut handle = window.handler.borrow_mut();
    let mut animations = window.animations.borrow_mut();

    //Send an action, if its handled remove it so a different window doesnt respond to it
    let result = handle.on_input(&mut animations, &mut helper, action);
    if result {
        //If the window responded to the action, set it to redraw
        window.window.request_redraw();
    }
    helper.resolve(&window.window);
    result
}

fn send_action_to_window(windows: &mut std::collections::HashMap<WindowId, YaffeWindow>, window_id: WindowId, action: &crate::Actions) {
    if let Some(window) = windows.get_mut(&window_id) {
        if handle_action(window, action) {
            return;
        }
    }

    for (id, window) in windows.iter_mut() {
        if *id == window_id {
            continue;
        } else if handle_action(window, action) {
            return;
        }
    }
}

fn check_for_updates(update_timer: &mut f32, delta_time: f32, queue: &ThreadSafeJobQueue) {
    //Check for updates once every hour if it hasnt been applied already
    if *update_timer != f32::MIN {
        *update_timer -= delta_time;
        if *update_timer < 0. {
            *update_timer = UPDATE_TIMER;

            let lock = queue.lock().log_and_panic();
            let mut queue = lock.borrow_mut();
            queue.send(Job::CheckUpdates).log("Unable to check for updates");
        }
    }
}
