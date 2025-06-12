use crate::{
    graphics::Graphics,
    input::ControllerInput,
    input::InputType,
    job_system::{Job, JobResult, JobResults, ThreadSafeJobQueue},
    logger::{LogEntry, PanicLogEntry},
    ui::AnimationManager,
    Actions, PhysicalSize,
};
use glutin::event::{Event, ModifiersState, VirtualKeyCode, WindowEvent};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{Fullscreen, WindowBuilder};
use speedy2d::dimen::Vector2;
use speedy2d::*;
use std::cell::RefCell;
use std::rc::Rc;
use std::time::Instant;

mod context_tracker;
use context_tracker::{ContextTracker, ContextWrapper};

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
    pub fn set_visibility(&mut self, visible: bool) {
        if visible {
            self.visible = Some(WindowVisibility::Visible);
        } else {
            self.visible = Some(WindowVisibility::Hide);
        }
    }

    pub fn resolve(self, window: &glutin::window::Window) {
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
    context_id: usize,
    renderer: GLRenderer,
    size: PhysicalSize,
    handler: std::rc::Rc<RefCell<dyn WindowHandler + 'static>>,
    graphics: RefCell<Graphics>,
    animations: RefCell<AnimationManager>,
    first_frame: RefCell<bool>,
}

fn create_best_context(
    window_builder: &WindowBuilder,
    event_loop: &EventLoop<()>,
) -> Option<glutin::WindowedContext<glutin::NotCurrent>> {
    for vsync in &[true, false] {
        for multisampling in &[8, 4, 2, 1, 0] {
            let mut windowed_context = glutin::ContextBuilder::new()
                .with_vsync(*vsync)
                .with_gl(glutin::GlRequest::Specific(glutin::Api::OpenGl, (2, 0)));

            if *multisampling > 1 {
                windowed_context = windowed_context.with_multisampling(*multisampling);
            }

            let result = windowed_context.build_windowed(window_builder.clone(), event_loop);

            match result {
                Ok(context) => {
                    return Some(context);
                }
                Err(err) => {
                    crate::logger::warn!("Failed to create context: {err:?}");
                }
            }
        }
    }

    None
}

fn create_window(
    windows: &mut std::collections::HashMap<glutin::window::WindowId, YaffeWindow>,
    event_loop: &EventLoop<()>,
    tracker: &mut ContextTracker,
    builder: WindowBuilder,
    handler: Rc<RefCell<impl WindowHandler + 'static>>,
    queue: ThreadSafeJobQueue,
) -> PhysicalSize {
    let windowed_context = create_best_context(&builder, event_loop).log_and_panic();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let id = windowed_context.window().id();
    let size = windowed_context.window().inner_size();
    let renderer = unsafe {
        GLRenderer::new_for_gl_context((size.width, size.height), |fn_name| {
            windowed_context.get_proc_address(fn_name) as *const _
        })
    }
    .unwrap();
    let context_id = tracker.insert(context_tracker::ContextCurrentWrapper::PossiblyCurrent(
        context_tracker::ContextWrapper::Windowed(windowed_context),
    ));

    let size = PhysicalSize::new(size.width as f32, size.height as f32);
    let window = YaffeWindow {
        context_id,
        renderer,
        size,
        handler,
        graphics: RefCell::new(Graphics::new(queue)),
        animations: RefCell::new(AnimationManager::new()),
        first_frame: RefCell::new(true),
    };
    windows.insert(id, window);
    size
}

fn do_and_redraw_window<F>(
    windows: &mut std::collections::HashMap<glutin::window::WindowId, YaffeWindow>,
    window_id: glutin::window::WindowId,
    context: &mut ContextTracker,
    mut action: F,
) where
    F: FnMut(&mut ContextWrapper<glutin::PossiblyCurrent>, &mut YaffeWindow),
{
    if let Some(window) = windows.get_mut(&window_id) {
        let context = context.get_current(window.context_id).unwrap();
        action(context, window);
        context.windowed().window().request_redraw();
    }
}

pub(crate) fn create_yaffe_windows(
    job_results: JobResults,
    queue: ThreadSafeJobQueue,
    mut gamepad: impl crate::input::PlatformGamepad + 'static,
    input_map: crate::input::InputMap<VirtualKeyCode, ControllerInput, Actions>,
    handler: Rc<RefCell<impl WindowHandler + 'static>>,
    overlay: Rc<RefCell<impl WindowHandler + 'static>>,
) -> ! {
    let el = EventLoop::new();

    let mut ct = context_tracker::ContextTracker::default();
    let mut windows = std::collections::HashMap::new();

    //https://github.com/rust-windowing/glutin/blob/master/glutin_examples/examples/multiwindow.rs
    let monitor = el.primary_monitor();
    let fullscreen = Some(Fullscreen::Borderless(monitor));
    let builder = WindowBuilder::new().with_title("Yaffe").with_fullscreen(fullscreen).with_visible(true);
    let size = create_window(&mut windows, &el, &mut ct, builder, handler, queue.clone());

    //Doing full size seems to make it fullscreen and it loses transparency
    let builder = WindowBuilder::new()
        .with_title("Overlay")
        .with_inner_size(glutin::dpi::PhysicalSize::new(size.x - 1., size.y - 1.))
        .with_position(glutin::dpi::PhysicalPosition::new(1i32, 1i32))
        .with_visible(false)
        .with_always_on_top(true)
        .with_transparent(true)
        .with_decorations(false);
    create_window(&mut windows, &el, &mut ct, builder, overlay, queue.clone());

    let mut delta_time = 0f32;
    let mut last_time = Instant::now();
    let mut mods = ModifiersState::empty();
    let mut update_timer = 0f32;
    let mut finished_jobs = vec![];

    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;

        crate::logger::trace!("Window event {event:?}");
        match event {
            Event::LoopDestroyed => *control_flow = ControlFlow::Exit,

            Event::WindowEvent { event, window_id } => match event {
                WindowEvent::CloseRequested => {
                    if let Some(window) = windows.get_mut(&window_id) {
                        *control_flow = ControlFlow::Exit;
                        window.handler.borrow_mut().on_stop();
                    }
                }

                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    do_and_redraw_window(&mut windows, window_id, &mut ct, |_, window| {
                        window.size = PhysicalSize::new(new_inner_size.width as f32, new_inner_size.height as f32);
                    });
                }

                WindowEvent::Resized(physical_size) => {
                    do_and_redraw_window(&mut windows, window_id, &mut ct, |context, window| {
                        window.size = PhysicalSize::new(physical_size.width as f32, physical_size.height as f32);

                        context.windowed().resize(physical_size);
                        window
                            .renderer
                            .set_viewport_size_pixels(Vector2::new(physical_size.width, physical_size.height));
                    });
                }

                WindowEvent::Focused(_focused) => {
                    do_and_redraw_window(&mut windows, window_id, &mut ct, |_, _| {});
                }

                WindowEvent::ModifiersChanged(state) => mods = state,

                WindowEvent::KeyboardInput { input, .. } => {
                    if glutin::event::ElementState::Released == input.state || input.virtual_keycode.is_none() {
                        return;
                    }

                    let action = if let Some(action) = input_map.get(input.virtual_keycode, None) {
                        action.clone()
                    } else if matches!(input.virtual_keycode, Some(VirtualKeyCode::V)) && mods.ctrl() {
                        if let Some(window) = windows.get_mut(&window_id) {
                            let context = ct.get_current(window.context_id).unwrap();

                            match crate::os::get_clipboard(context.windowed().window()) {
                                Some(clip) => crate::Actions::KeyPress(InputType::Paste(clip)),
                                _ => return,
                            }
                        } else {
                            return;
                        }
                    } else {
                        crate::Actions::KeyPress(InputType::Key(input.virtual_keycode.unwrap()))
                    };

                    for (_, window) in windows.iter_mut() {
                        if send_action_to_window(window, &mut ct, &action) {
                            return;
                        }
                    }
                }

                WindowEvent::ReceivedCharacter(c) => {
                    if c.is_control() {
                        return;
                    }
                    let action = &crate::Actions::KeyPress(InputType::Char(c));

                    for (_, window) in windows.iter_mut() {
                        if send_action_to_window(window, &mut ct, action) {
                            return;
                        }
                    }
                }

                _ => {}
            },

            Event::RedrawRequested(id) => {
                if let Some(window) = windows.get_mut(&id) {
                    let context = ct.get_current(window.context_id).unwrap();

                    let scale = context.windowed().window().scale_factor() as f32;
                    let size = PhysicalSize::new(window.size.x, window.size.y);
                    let mut handle = window.handler.borrow_mut();
                    let mut window_graphics = window.graphics.borrow_mut();
                    let mut first_frame = window.first_frame.borrow_mut();
                    window.renderer.draw_frame(|graphics| {
                        unsafe {
                            window_graphics.set_frame(graphics, scale, size, delta_time);
                        }

                        if *first_frame {
                            handle.on_init(&mut window_graphics);
                            *first_frame = false;
                        }

                        handle.on_frame_begin(&mut window_graphics, &mut finished_jobs);
                        if !handle.on_frame(&mut window_graphics) {
                            *control_flow = ControlFlow::Exit;
                            handle.on_stop();
                        }
                    });
                    context.windowed().swap_buffers().unwrap();
                }
                finished_jobs.clear();
            }

            Event::MainEventsCleared => {
                //We need to calc delta time here because its always called
                //RedrawRequested is only called conditionally so we could skip many frames
                let now = Instant::now();
                delta_time = (now - last_time).as_millis() as f32 / 1000.;
                last_time = now;

                //Get controller input
                gamepad.update(0).log("Unable to get controller input");

                //Convert our input to actions we will propogate through the UI
                let mut actions = crate::input::input_to_action(&input_map, &mut gamepad);

                check_for_updates(&mut update_timer, delta_time, &queue);

                // Get results from any completed jobs
                while let Ok(result) = job_results.try_recv() {
                    finished_jobs.push(result);
                }
                // Process our "system" jobs
                crate::job_system::process_results(
                    &mut finished_jobs,
                    |j| matches!(j, JobResult::CheckUpdates(_)),
                    |result| {
                        if let JobResult::CheckUpdates(applied) = result {
                            if applied {
                                update_timer = f32::MIN;
                            }
                        }
                    },
                );
                let jobs_completed = !finished_jobs.is_empty();

                for (_, window) in windows.iter_mut() {
                    //Send an action, if its handled remove it so a different window doesnt respond to it
                    actions.retain(|action| !send_action_to_window(window, &mut ct, action));

                    //Raise fixed update every frame so we can do things even if redraws arent happening
                    let mut handle = window.handler.borrow_mut();
                    let context = ct.get_current(window.context_id).unwrap();

                    // Fixed update
                    let mut helper = WindowHelper { visible: None };
                    let fixed_update = handle.on_fixed_update(&mut helper);
                    helper.resolve(context.windowed().window());

                    // Process animations
                    let mut animations = window.animations.borrow_mut();
                    let root = handle.get_ui();
                    animations.process(root, delta_time);
                    let is_dirty = animations.is_dirty();

                    if fixed_update || jobs_completed || is_dirty {
                        context.windowed().window().request_redraw();
                    }
                }
            }

            _ => {}
        }
    });
}

fn send_action_to_window(
    window: &mut YaffeWindow,
    ct: &mut context_tracker::ContextTracker,
    action: &crate::Actions,
) -> bool {
    let mut helper = WindowHelper { visible: None };
    let context = ct.get_current(window.context_id).unwrap();
    let mut handle = window.handler.borrow_mut();
    let mut animations = window.animations.borrow_mut();

    //Send an action, if its handled remove it so a different window doesnt respond to it
    let result = handle.on_input(&mut animations, &mut helper, action);
    if result {
        //If the window responded to the action, set it to redraw
        context.windowed().window().request_redraw();
    }
    helper.resolve(context.windowed().window());

    result
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
