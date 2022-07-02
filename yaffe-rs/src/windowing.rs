use speedy2d::*;
use speedy2d::dimen::Vector2;
use glutin::event::{Event, WindowEvent, VirtualKeyCode, ModifiersState};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{Fullscreen, WindowBuilder};
use crate::{input::ControllerInput, PhysicalSize, input::InputType, Actions, logger::LogEntry};
use std::time::Instant;
use std::rc::Rc;
use std::cell::RefCell;

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
        if visible { self.visible = Some(WindowVisibility::Visible); }
        else { self.visible = Some(WindowVisibility::Hide); }
    }

    pub fn resolve(self, window: &glutin::window::Window) {
        match self.visible {
            Some(WindowVisibility::Hide) => window.set_visible(false),
            Some(WindowVisibility::Visible) => window.set_visible(true),
            None => {},
        }
    }
}

pub(crate) trait WindowHandler {
    fn on_fixed_update(&mut self, _: &mut WindowHelper, _: f32) -> bool { false }
    fn on_frame(&mut self, graphics: &mut Graphics2D, delta_time: f32, size: PhysicalSize, scale_factor: f32) -> bool;
    fn on_input(&mut self, helper: &mut WindowHelper, action: &crate::Actions) -> bool;
    fn on_resize(&mut self, _: u32, _: u32) { }
    fn on_stop(&mut self) { }
    fn is_window_dirty(&self) -> bool {
        false
    }
}

struct YaffeWindow {
    context_id: usize,
    renderer: GLRenderer,
    size: PhysicalSize,
    handler: std::rc::Rc<RefCell<dyn WindowHandler + 'static>>,
}

fn create_best_context(window_builder: &WindowBuilder, event_loop: &EventLoop<()>) -> Option<glutin::WindowedContext<glutin::NotCurrent>> {
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
                Ok(context) => { return Some(context); }
                Err(err) => {
                    crate::logger::log_entry!(crate::logger::LogTypes::Warning, "Failed to create context: {:?}", err);
                }
            }
        }
    }

    None
}

fn create_window(windows: &mut std::collections::HashMap<glutin::window::WindowId, YaffeWindow>,
                 event_loop: &EventLoop<()>, 
                 tracker: &mut context_tracker::ContextTracker, 
                 builder: WindowBuilder,
                 handler: Rc<RefCell<impl WindowHandler + 'static>>) -> PhysicalSize {

    use crate::logger::PanicLogEntry;
    let windowed_context = create_best_context(&builder, &event_loop).log_and_panic();
    let windowed_context = unsafe { windowed_context.make_current().unwrap() };

    let id = windowed_context.window().id();
    let size = windowed_context.window().inner_size();
    let renderer = unsafe {
        GLRenderer::new_for_gl_context((size.width, size.height), |fn_name| {
            windowed_context.get_proc_address(fn_name) as *const _
        })
    }.unwrap();
    let context_id = tracker.insert(context_tracker::ContextCurrentWrapper::PossiblyCurrent(
        context_tracker::ContextWrapper::Windowed(windowed_context),
    ));

    let size = PhysicalSize::new(size.width as f32, size.height as f32);
    let window = YaffeWindow { context_id, renderer, size, handler};
    windows.insert(id, window);
    size
}

pub(crate) fn create_yaffe_windows(notify: std::sync::mpsc::Receiver<u8>,
                                   mut gamepad: impl crate::input::PlatformGamepad + 'static,
                                   input_map: crate::input::InputMap<VirtualKeyCode, ControllerInput, Actions>,
                                   handler: Rc<RefCell<impl WindowHandler + 'static>>,
                                   overlay: Rc<RefCell<impl WindowHandler + 'static>>) -> ! {
    let el = EventLoop::new();

    let mut ct = context_tracker::ContextTracker::default();
    let mut windows = std::collections::HashMap::new();

    //https://github.com/rust-windowing/glutin/blob/master/glutin_examples/examples/multiwindow.rs
    let monitor = el.primary_monitor();
    let fullscreen = Some(Fullscreen::Borderless(monitor));
    let builder = WindowBuilder::new()
        .with_title("Yaffe")  
        .with_fullscreen(fullscreen.clone())
        .with_visible(true);
    let size = create_window(&mut windows, &el, &mut ct, builder, handler);

    //Doing full size seems to make it fullscreen and it loses transparency
    let builder = WindowBuilder::new()
        .with_title("Overlay")
        .with_inner_size(glutin::dpi::PhysicalSize::new(size.x - 1., size.y - 1.)) 
        .with_position(glutin::dpi::PhysicalPosition::new(1i32, 1i32))
        .with_visible(false)
        .with_always_on_top(true)
        .with_transparent(true)
        .with_decorations(false);
    create_window(&mut windows, &el, &mut ct, builder, overlay);

    let mut delta_time = 0f32;
    let mut last_time = Instant::now();
    let mut mods = ModifiersState::empty();
    
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        
        match event {
            Event::LoopDestroyed => *control_flow = ControlFlow::Exit,

            Event::WindowEvent { event, window_id } => match event {
                WindowEvent::CloseRequested => {
                    crate::logger::log_entry!(crate::logger::LogTypes::Fine, "Closing window");

                    if let Some(window) = windows.get_mut(&window_id) {
                        *control_flow = ControlFlow::Exit;
                        window.handler.borrow_mut().on_stop();
                    }
                },

                WindowEvent::ScaleFactorChanged { new_inner_size, .. } => {
                    crate::logger::log_entry!(crate::logger::LogTypes::Fine, "Scale factor changed, redrawing");

                    if let Some(window) = windows.get_mut(&window_id) {
                        let context = ct.get_current(window.context_id).unwrap();

                        window.size = PhysicalSize::new(new_inner_size.width as f32, new_inner_size.height as f32);

                        window.handler.borrow_mut().on_resize(new_inner_size.width, new_inner_size.height);
                        context.windowed().window().request_redraw();
                    }
                },

                WindowEvent::Resized(physical_size) => {
                    crate::logger::log_entry!(crate::logger::LogTypes::Fine, "Window resized, redrawing");

                    if let Some(window) = windows.get_mut(&window_id) {
                        let context = ct.get_current(window.context_id).unwrap();

                        window.size = PhysicalSize::new(physical_size.width as f32, physical_size.height as f32);

                        context.windowed().resize(physical_size);
                        window.renderer.set_viewport_size_pixels(Vector2::new(physical_size.width, physical_size.height));
                        window.handler.borrow_mut().on_resize(physical_size.width, physical_size.height);

                        context.windowed().window().request_redraw();
                    }
                },

                WindowEvent::Focused(_focused) => {
                    crate::logger::log_entry!(crate::logger::LogTypes::Fine, "Focus changed, redrawing");

                    if let Some(window) = windows.get_mut(&window_id) {
                        let context = ct.get_current(window.context_id).unwrap();
                        context.windowed().window().request_redraw();
                    }
                },

                WindowEvent::ModifiersChanged(state) => mods = state,

                WindowEvent::KeyboardInput { input, .. } => {
                    if let glutin::event::ElementState::Released = input.state { return; }
                    if let None = input.virtual_keycode { return; }

                    let action = if let Some(action) = input_map.get(input.virtual_keycode, None) {
                        action.clone()
                    } else if matches!(input.virtual_keycode, Some(VirtualKeyCode::V)) && mods.ctrl() {
                            if let Some(window) = windows.get_mut(&window_id) {
                                let context = ct.get_current(window.context_id).unwrap();

                                match crate::platform_layer::get_clipboard(context.windowed().window()) {
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
                        if send_action_to_window(window, &mut ct, &action) { return; }
                    }
                },

                WindowEvent::ReceivedCharacter(c) => {
                    if c.is_control() { return; }
                    let action = &crate::Actions::KeyPress(InputType::Char(c));

                    for (_, window) in windows.iter_mut() {
                        if send_action_to_window(window, &mut ct, action) { return; }
                    }
                }

                _ => {}
            },

            Event::RedrawRequested(id) => {
                if let Some(window) = windows.get_mut(&id) {
                    let context = ct.get_current(window.context_id).unwrap();

                    let scale = context.windowed().window().scale_factor() as f32;
                    let size = PhysicalSize::new(window.size.x as f32, window.size.y as f32);
                    let mut handle = window.handler.borrow_mut();
                    window.renderer.draw_frame(|graphics| {
                        if !handle.on_frame(graphics, delta_time, size, scale) {
                            *control_flow = ControlFlow::Exit;
                            handle.on_stop();
                        }
                    });
                    context.windowed().swap_buffers().unwrap();
                }
            },

            Event::MainEventsCleared => {
                //We need to calc delta time here because its always called
                //RedrawRequested is only called conditionally so we could skip many frames
                let now = Instant::now();
                delta_time = (now - last_time).as_millis() as f32 / 1000.;
                last_time = now;

                //Get controller input
                gamepad.update(0).log("Unable to get controller input");

                //Convert our input to actions we will propogate through the UI
                let mut actions = input_to_action(&input_map, &mut gamepad);
                let asset_loaded = notify.try_recv().is_ok();

                for (_, window) in windows.iter_mut() {
                    //Send an action, if its handled remove it so a different window doesnt respond to it
                    let mut handled_actions = Vec::with_capacity(actions.len());
                    for action in actions.iter() {
                        if send_action_to_window(window, &mut ct, action) {
                            handled_actions.push(action.clone());
                        }
                    }
                    for action in handled_actions { actions.remove(&action); }

                    //Raise fixed update every frame so we can do things even if redraws arent happening
                    let mut handle = window.handler.borrow_mut();
                    let context = ct.get_current(window.context_id).unwrap();
                    
                    let mut helper = WindowHelper { visible: None, };
                    let fixed_update = handle.on_fixed_update(&mut helper, delta_time);
                    helper.resolve(&context.windowed().window());

                    if fixed_update || asset_loaded || handle.is_window_dirty() {
                        context.windowed().window().request_redraw();
                    }
                }
            }

            _ => {}
        }
    });
}

fn send_action_to_window(window: &mut YaffeWindow, 
                         ct: &mut context_tracker::ContextTracker,
                         action: &crate::Actions) -> bool {
    let mut helper = WindowHelper { visible: None, };
    let context = ct.get_current(window.context_id).unwrap();
    let mut handle = window.handler.borrow_mut();

    //Send an action, if its handled remove it so a different window doesnt respond to it
    let result = handle.on_input(&mut helper, action);
    if result { 
        //If the window responded to the action, set it to redraw
        context.windowed().window().request_redraw();
    } 
    helper.resolve(&context.windowed().window());

    result
}

fn input_to_action(input_map: &crate::input::InputMap<VirtualKeyCode, ControllerInput, Actions>, 
                   input: &mut dyn crate::input::PlatformGamepad) -> std::collections::HashSet<Actions> {

    let mut result = std::collections::HashSet::new();
    for g in input.get_gamepad() {
        if let Some(action) = input_map.get(None, Some(g)) {
            result.insert(action.clone());
        } else {
            result.insert(Actions::KeyPress(InputType::Char(g as u8 as char)));
        }
    }

    result
}

mod context_tracker {
    use glutin::{
        self, ContextCurrentState, ContextError, NotCurrent, PossiblyCurrent,
        WindowedContext,
    };
    use takeable_option::Takeable;

    pub enum ContextWrapper<T: ContextCurrentState> {
        Windowed(WindowedContext<T>),
    }

    impl<T: ContextCurrentState> ContextWrapper<T> {
        pub fn windowed(&mut self) -> &mut WindowedContext<T> {
            match self {
                ContextWrapper::Windowed(ref mut ctx) => ctx,
            }
        }

        fn map<T2: ContextCurrentState, FW>(
            self,
            fw: FW,
        ) -> Result<ContextWrapper<T2>, (Self, ContextError)>
        where
            FW: FnOnce(WindowedContext<T>) -> Result<WindowedContext<T2>, (WindowedContext<T>, ContextError)>,
        {
            match self {
                ContextWrapper::Windowed(ctx) => match fw(ctx) {
                    Ok(ctx) => Ok(ContextWrapper::Windowed(ctx)),
                    Err((ctx, err)) => Err((ContextWrapper::Windowed(ctx), err)),
                },
            }
        }
    }

    pub enum ContextCurrentWrapper {
        PossiblyCurrent(ContextWrapper<PossiblyCurrent>),
        NotCurrent(ContextWrapper<NotCurrent>),
    }

    impl ContextCurrentWrapper {
        fn map_possibly<F>(self, f: F) -> Result<Self, (Self, ContextError)>
        where
            F: FnOnce(
                ContextWrapper<PossiblyCurrent>,
            ) -> Result<ContextWrapper<NotCurrent>, (ContextWrapper<PossiblyCurrent>, ContextError)>,
        {
            match self {
                ret @ ContextCurrentWrapper::NotCurrent(_) => Ok(ret),
                ContextCurrentWrapper::PossiblyCurrent(ctx) => match f(ctx) {
                    Ok(ctx) => Ok(ContextCurrentWrapper::NotCurrent(ctx)),
                    Err((ctx, err)) => Err((ContextCurrentWrapper::PossiblyCurrent(ctx), err)),
                },
            }
        }

        fn map_not<F>(self, f: F) -> Result<Self, (Self, ContextError)>
        where
            F: FnOnce(
                ContextWrapper<NotCurrent>,
            ) -> Result<ContextWrapper<PossiblyCurrent>, (ContextWrapper<NotCurrent>, ContextError)>,
        {
            match self {
                ret @ ContextCurrentWrapper::PossiblyCurrent(_) => Ok(ret),
                ContextCurrentWrapper::NotCurrent(ctx) => match f(ctx) {
                    Ok(ctx) => Ok(ContextCurrentWrapper::PossiblyCurrent(ctx)),
                    Err((ctx, err)) => Err((ContextCurrentWrapper::NotCurrent(ctx), err)),
                },
            }
        }
    }

    pub type ContextId = usize;
    #[derive(Default)]
    pub struct ContextTracker {
        current: Option<ContextId>,
        others: Vec<(ContextId, Takeable<ContextCurrentWrapper>)>,
        next_id: ContextId,
    }

    impl ContextTracker {
        pub fn insert(&mut self, ctx: ContextCurrentWrapper) -> ContextId {
            let id = self.next_id;
            self.next_id += 1;

            if let ContextCurrentWrapper::PossiblyCurrent(_) = ctx {
                if let Some(old_current) = self.current {
                    unsafe {
                        self.modify(old_current, |ctx| {
                            ctx.map_possibly(|ctx| {
                                ctx.map(|ctx| Ok(ctx.treat_as_not_current()), )
                            })
                        })
                        .unwrap()
                    }
                }
                self.current = Some(id);
            }

            self.others.push((id, Takeable::new(ctx)));
            id
        }

        fn modify<F>(&mut self, id: ContextId, f: F) -> Result<(), ContextError>
        where
            F: FnOnce(ContextCurrentWrapper) -> Result<ContextCurrentWrapper, (ContextCurrentWrapper, ContextError)>,
        {
            let this_index = self.others.binary_search_by(|(sid, _)| sid.cmp(&id)).unwrap();

            let this_context = Takeable::take(&mut self.others[this_index].1);

            match f(this_context) {
                Err((ctx, err)) => {
                    self.others[this_index].1 = Takeable::new(ctx);
                    Err(err)
                }
                Ok(ctx) => {
                    self.others[this_index].1 = Takeable::new(ctx);
                    Ok(())
                }
            }
        }

        pub fn get_current(
            &mut self,
            id: ContextId,
        ) -> Result<&mut ContextWrapper<PossiblyCurrent>, ContextError> {
            unsafe {
                let this_index = self.others.binary_search_by(|(sid, _)| sid.cmp(&id)).unwrap();
                if Some(id) != self.current {
                    let old_current = self.current.take();

                    if let Err(err) = self.modify(id, |ctx| {
                        ctx.map_not(|ctx| {
                            ctx.map(|ctx| ctx.make_current())
                        })
                    }) {
                        // Oh noes, something went wrong
                        // Let's at least make sure that no context is current.
                        if let Some(old_current) = old_current {
                            if let Err(err2) = self.modify(old_current, |ctx| {
                                ctx.map_possibly(|ctx| {
                                    ctx.map(|ctx| ctx.make_not_current(), )
                                })
                            }) {
                                panic!("Could not `make_current` nor `make_not_current`, {:?}, {:?}", err, err2);
                            }
                        }

                        if let Err(err2) = self.modify(id, |ctx| {
                            ctx.map_possibly(|ctx| {
                                ctx.map(|ctx| ctx.make_not_current())
                            })
                        }) {
                            panic!("Could not `make_current` nor `make_not_current`, {:?}, {:?}", err, err2);
                        }

                        return Err(err);
                    }

                    self.current = Some(id);

                    if let Some(old_current) = old_current {
                        self.modify(old_current, |ctx| {
                            ctx.map_possibly(|ctx| {
                                ctx.map(|ctx| Ok(ctx.treat_as_not_current()), )
                            })
                        })
                        .unwrap();
                    }
                }

                match *self.others[this_index].1 {
                    ContextCurrentWrapper::PossiblyCurrent(ref mut ctx) => Ok(ctx),
                    ContextCurrentWrapper::NotCurrent(_) => panic!(),
                }
            }
        }
    }
}