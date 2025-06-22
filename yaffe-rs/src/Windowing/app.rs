use std::cell::RefCell;
use std::collections::{HashMap, HashSet};
use std::num::NonZeroU32;
use std::time::Instant;

use speedy2d::dimen::Vector2;
use speedy2d::GLRenderer;

use glutin::config::ConfigTemplateBuilder;
use glutin::context::{ContextApi, ContextAttributesBuilder, Version};
use glutin::display::GetGlDisplay;
use glutin::prelude::*;
use glutin::surface::SurfaceAttributesBuilder;

use glutin_winit::{DisplayBuilder, GlWindow};

use winit::application::ApplicationHandler;
use winit::event::{ElementState, WindowEvent};
use winit::event_loop::ActiveEventLoop;
use winit::keyboard::{KeyCode, ModifiersState, PhysicalKey};
use winit::raw_window_handle::HasWindowHandle;
use winit::window::{Fullscreen, WindowId};

use super::{AnimationManager, InputType, JobResults, WindowHelper, WindowInfo, YaffeWindow};
use crate::input::{Actions, ControllerInput, InputMap, PlatformGamepad};
use crate::job_system::{JobResult, ThreadSafeJobQueue};
use crate::logger::LogEntry;
use crate::Graphics;

static mut CURRENT_WINDOW_ID: WindowId = WindowId::dummy();
pub fn get_current_window() -> WindowId {
    #[allow(static_mut_refs)]
    return unsafe { CURRENT_WINDOW_ID.clone() };
}

pub struct App {
    windows: HashMap<WindowId, YaffeWindow>,
    modifiers: Option<ModifiersState>,
    input_map: InputMap<KeyCode, ControllerInput, Actions>,
    queue: ThreadSafeJobQueue,
    window_infos: Vec<WindowInfo>,
    finished_jobs: Vec<(Option<WindowId>, JobResult)>,
    delta_time: f32,
    update_timer: f32,
    last_time: Instant,
    job_results: JobResults,
    gamepad: Box<dyn PlatformGamepad + 'static>,
    handled_actions: HashSet<Actions>,
}

impl App {
    pub fn new(
        input_map: InputMap<KeyCode, ControllerInput, Actions>,
        queue: ThreadSafeJobQueue,
        window_infos: Vec<WindowInfo>,
        job_results: JobResults,
        gamepad: impl PlatformGamepad + 'static,
    ) -> Self {
        Self {
            windows: HashMap::new(),
            modifiers: None,
            input_map,
            queue,
            window_infos,
            finished_jobs: vec![],
            delta_time: 0.,
            update_timer: 0.,
            last_time: Instant::now(),
            job_results,
            gamepad: Box::new(gamepad),
            handled_actions: HashSet::new(),
        }
    }

    fn create_window(
        event_loop: &ActiveEventLoop,
        info: WindowInfo,
        queue: ThreadSafeJobQueue,
    ) -> (WindowId, YaffeWindow) {
        for multisampling in &[16, 8, 4, 2, 1, 0] {
            let mut template = ConfigTemplateBuilder::new().with_transparency(true);

            if *multisampling > 1 {
                template = template.with_multisampling(*multisampling);
            }

            let display_builder = DisplayBuilder::new().with_window_attributes(Some(info.attributes.clone()));
            let result = display_builder.build(event_loop, template, |mut configs| configs.next().unwrap());

            let (window, gl_config) = match result {
                Ok((Some(window), config)) => {
                    log::info!("Window created");
                    (window, config)
                }
                Ok((None, _)) => {
                    log::info!("Failed with null window");
                    continue;
                }
                Err(err) => {
                    log::info!("Failed with error: {err:?}");
                    continue;
                }
            };

            let gl_display = gl_config.display();

            let context_attributes = ContextAttributesBuilder::new()
                .with_context_api(ContextApi::OpenGl(Some(Version::new(2, 0))))
                .build(Some(window.window_handle().unwrap().into()));

            let context = match unsafe { gl_display.create_context(&gl_config, &context_attributes) } {
                Ok(context) => context,
                Err(err) => {
                    log::info!("Failed to create context with error: {err:?}");
                    continue;
                }
            };

            let attrs = window.build_surface_attributes(SurfaceAttributesBuilder::default()).unwrap();

            let gl_surface = match unsafe { gl_config.display().create_window_surface(&gl_config, &attrs) } {
                Ok(surface) => surface,
                Err(err) => {
                    log::info!("Failed to finalize surface with error: {err:?}");
                    continue;
                }
            };

            let gl_context = match context.make_current(&gl_surface) {
                Ok(context) => context,
                Err(err) => {
                    log::info!("Failed to make context current with error: {err:?}");
                    continue;
                }
            };

            let size = window.inner_size();
            let mut renderer = unsafe {
                GLRenderer::new_for_gl_context((size.width, size.height), |fn_name| {
                    gl_context.display().get_proc_address(std::ffi::CString::new(fn_name).unwrap().as_c_str())
                        as *const _
                })
                .unwrap()
            };

            // Call window init
            let mut handler = info.handler.borrow_mut();
            let mut window_graphics = Graphics::new(queue);
            let size = crate::PhysicalSize::new(size.width as f32, size.height as f32);
            renderer.draw_frame(|graphics| {
                unsafe {
                    window_graphics.set_frame(graphics, 1., size);
                }
                handler.on_init(&mut window_graphics);
            });

            let window_id = window.id();
            return (
                window_id,
                YaffeWindow {
                    window,
                    gl_context,
                    gl_surface,
                    renderer,
                    size,
                    handler: info.handler.clone(),
                    graphics: RefCell::new(window_graphics),
                    animations: RefCell::new(AnimationManager::new()),
                },
            );
        }
        panic!("Unable to create window")
    }

    fn get_processed_jobs(&mut self, window_id: Option<WindowId>) -> Vec<JobResult> {
        self.finished_jobs.extract_if(.., |(id, _)| *id == window_id).map(|(_, r)| r).collect()
    }
}

impl ApplicationHandler for App {
    fn resumed(&mut self, event_loop: &ActiveEventLoop) {
        let monitor = event_loop.primary_monitor();
        let fullscreen = Some(Fullscreen::Borderless(monitor));

        for mut i in self.window_infos.drain(..) {
            if i.fullscreen {
                i.attributes = i.attributes.with_fullscreen(fullscreen.clone());
            }
            let (id, window) = Self::create_window(event_loop, i, self.queue.clone());
            self.windows.insert(id, window);
        }
    }

    fn window_event(&mut self, event_loop: &ActiveEventLoop, window_id: WindowId, event: WindowEvent) {
        unsafe { CURRENT_WINDOW_ID = window_id.clone() }
        match event {
            WindowEvent::CloseRequested => {
                for window in self.windows.values() {
                    window.handler.borrow_mut().on_stop();
                }
                event_loop.exit();
            }

            WindowEvent::Resized(physical_size) => {
                if let Some(window) = self.windows.get_mut(&window_id) {
                    window.size = crate::PhysicalSize::new(physical_size.width as f32, physical_size.height as f32);

                    window.gl_surface.resize(
                        &window.gl_context,
                        NonZeroU32::new(physical_size.width.max(1)).unwrap(),
                        NonZeroU32::new(physical_size.height.max(1)).unwrap(),
                    );
                    window.renderer.set_viewport_size_pixels(Vector2::new(physical_size.width, physical_size.height));
                    window.window.request_redraw();
                }
            }

            WindowEvent::ScaleFactorChanged { .. } | WindowEvent::Focused(true) => {
                if let Some(window) = self.windows.get(&window_id) {
                    window.window.request_redraw()
                }
            }

            WindowEvent::ModifiersChanged(state) => self.modifiers = Some(state.state()),

            WindowEvent::KeyboardInput { event, .. } => {
                let PhysicalKey::Code(keycode) = event.physical_key else {
                    return;
                };
                if ElementState::Released == event.state {
                    return;
                }

                let text = event.logical_key.to_text().map(|s| s.to_string());
                let action = Actions::KeyPress(InputType::Key(keycode, text, self.modifiers));
                if let Some(window) = self.windows.get_mut(&window_id) {
                    super::handle_action(window, &action);
                }

                let Some(action) = self.input_map.get(Some(keycode), None) else {
                    return;
                };

                // Only handle each action once per frame. This fixes issues where the actions will trigger once for overlay and once for main, causing double actions
                if self.handled_actions.insert(action.clone()) {
                    super::send_action_to_window(&mut self.windows, window_id, &action);
                }
            }

            WindowEvent::RedrawRequested => {
                self.handled_actions.clear();
                let jobs = self.get_processed_jobs(Some(window_id));
                if let Some(window) = self.windows.get_mut(&window_id) {
                    let scale = window.window.scale_factor() as f32;
                    let size = crate::PhysicalSize::new(window.size.x, window.size.y);
                    let mut handle = window.handler.borrow_mut();
                    let mut window_graphics = window.graphics.borrow_mut();

                    window.gl_context.make_current(&window.gl_surface).unwrap();
                    window.renderer.draw_frame(|graphics| {
                        unsafe {
                            window_graphics.set_frame(graphics, scale, size);
                        }

                        handle.on_frame_begin(&mut window_graphics, jobs);
                        if !handle.on_frame(&mut window_graphics) {
                            event_loop.exit();
                            handle.on_stop();
                        }
                    });
                    window.gl_surface.swap_buffers(&window.gl_context).unwrap();
                }
            }
            _ => {}
        }
    }

    fn about_to_wait(&mut self, _event_loop: &ActiveEventLoop) {
        // Request redraw for all windows
        //We need to calc delta time here because its always called
        //RedrawRequested is only called conditionally so we could skip many frames
        let now = Instant::now();
        self.delta_time = (now - self.last_time).as_millis() as f32 / 1000.;
        self.last_time = now;

        //Get controller input
        self.gamepad.update(0).log("Unable to get controller input");

        //Convert our input to actions we will propogate through the UI
        let mut gamepad_actions = crate::input::input_to_action(&self.input_map, &mut *self.gamepad);

        super::check_for_updates(&mut self.update_timer, self.delta_time, &self.queue);

        // Get results from any completed jobs
        while let Ok(result) = self.job_results.try_recv() {
            self.finished_jobs.push(result);
        }
        // Process our "system" jobs
        let jobs = self.get_processed_jobs(None);
        process_system_jobs(&mut self.update_timer, jobs);

        let jobs_completed = !self.finished_jobs.is_empty();

        for (_, window) in self.windows.iter_mut() {
            //Send an action, if its handled remove it so a different window doesnt respond to it
            gamepad_actions.retain(|action| !super::handle_action(window, action));

            //Raise fixed update every frame so we can do things even if redraws arent happening
            let mut handle = window.handler.borrow_mut();
            let mut animations = window.animations.borrow_mut();

            // Fixed update
            let mut helper = WindowHelper::new();
            let fixed_update = handle.on_fixed_update(&mut animations, self.delta_time, &mut helper);
            helper.resolve(&window.window);

            let is_dirty = animations.is_dirty();
            if fixed_update || jobs_completed || is_dirty {
                window.window.request_redraw();
            }
        }
    }
}

fn process_system_jobs(timer: &mut f32, job_results: Vec<JobResult>) {
    for r in job_results {
        match r {
            JobResult::CheckUpdates(true) => *timer = f32::MIN,
            _ => {}
        }
    }
}
