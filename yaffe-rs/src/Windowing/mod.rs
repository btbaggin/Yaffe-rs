use crate::{
    graphics::Graphics,
    input::{ControllerInput, InputMap, InputType, PlatformGamepad},
    job_system::{Job, JobResult, JobResults, ThreadSafeJobQueue},
    ui::AnimationManager,
    Actions, PhysicalSize,
};
use std::cell::RefCell;

use glutin::surface::WindowSurface;

use winit::event_loop::{ControlFlow, EventLoop};
use winit::keyboard::KeyCode;
use winit::window::{Window, WindowAttributes, WindowId};

use speedy2d::GLRenderer;

mod app;
pub use app::get_current_window;

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
    pub fn new() -> WindowHelper { WindowHelper { visible: None } }

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
    fn on_fixed_update(
        &mut self,
        animations: &mut AnimationManager,
        delta_time: f32,
        helper: &mut WindowHelper,
    ) -> bool;
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

pub struct WindowInfo {
    handler: std::rc::Rc<RefCell<dyn WindowHandler + 'static>>,
    attributes: WindowAttributes,
    fullscreen: bool,
}
impl WindowInfo {
    pub fn new(
        handler: std::rc::Rc<RefCell<dyn WindowHandler + 'static>>,
        attributes: WindowAttributes,
        fullscreen: bool,
    ) -> WindowInfo {
        WindowInfo { handler, attributes, fullscreen }
    }
}

pub fn run_app(
    input_map: InputMap<KeyCode, ControllerInput, Actions>,
    queue: ThreadSafeJobQueue,
    window_infos: Vec<WindowInfo>,
    job_results: JobResults,
    gamepad: impl PlatformGamepad + 'static,
) {
    let el = EventLoop::new().unwrap();
    el.set_control_flow(ControlFlow::Poll);

    let mut app = app::App::new(input_map, queue, window_infos, job_results, gamepad);
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

fn send_action_to_window(
    windows: &mut std::collections::HashMap<WindowId, YaffeWindow>,
    window_id: WindowId,
    action: &crate::Actions,
) {
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

            queue.start_job(Job::CheckUpdates);
        }
    }
}
