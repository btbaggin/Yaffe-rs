use speedy2d::*;
use speedy2d::dimen::Vector2;
use glutin::event::{Event, WindowEvent, VirtualKeyCode};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{Fullscreen, WindowBuilder};
use crate::{V2, input::ControllerInput};
use std::time::Instant;
use std::rc::Rc;
use std::cell::RefCell;

pub trait Rect {
    fn left(&self) -> f32;
    fn right(&self) -> f32;
    fn top(&self) -> f32;
    fn bottom(&self) -> f32;
    fn point_and_size(pos: V2, size: V2) -> Self;
}
impl Rect for speedy2d::shape::Rectangle {
    fn left(&self) -> f32 { self.top_left().x }
    fn right(&self) -> f32 { self.bottom_right().x }
    fn top(&self) -> f32 { self.top_left().y }
    fn bottom(&self) -> f32 { self.bottom_right().y }
    fn point_and_size(pos: V2, size: V2) -> Self { speedy2d::shape::Rectangle::new(pos, pos + size) }
}

pub trait Transparent {
    fn with_alpha(&self, alpha: f32) -> Self;
}
impl Transparent for speedy2d::color::Color {
    fn with_alpha(&self, alpha: f32) -> Self {
        speedy2d::color::Color::from_rgba(self.r(), self.g(), self.b(), alpha)
    }
}

#[repr(u8)]
enum WindowVisibility {
    Visible,
    Hide,
}

#[repr(u8)]
enum ModalFileAction {
    OpenFile,
    OpenDirectory,
}

pub struct WindowHelper {
    visible: Option<WindowVisibility>,
    file_action: Option<ModalFileAction>,
}

impl WindowHelper {
    pub fn set_visibility(&mut self, visible: bool) {
        if visible { self.visible = Some(WindowVisibility::Visible); }
        else { self.visible = Some(WindowVisibility::Hide); }
    }

    pub fn open_file(&mut self) {
        self.file_action = Some(ModalFileAction::OpenFile);
    }
    pub fn open_directory(&mut self) {
        self.file_action = Some(ModalFileAction::OpenDirectory);
    }

    pub fn resolve(self, window: &glutin::window::Window) {
        match self.visible {
            Some(WindowVisibility::Hide) => window.set_visible(false),
            Some(WindowVisibility::Visible) => window.set_visible(true),
            None => {},
        }

        match self.file_action {
            Some(ModalFileAction::OpenFile) =>  { Some(1) } //TODO state.win.handle.open_file(druid_shell::FileDialogOptions::new()),
            Some(ModalFileAction::OpenDirectory) => { None
                //TODO
                // let options = druid_shell::FileDialogOptions::new();
                // let options = options.select_directories();
                // state.win.handle.open_file(options)
            }
            None => None,
        };
    }
}

pub(crate) trait WindowHandler {
    fn on_start(&mut self) { }
    fn on_fixed_update(&mut self, _: &mut WindowHelper) { }
    fn on_frame(&mut self, graphics: &mut Graphics2D, delta_time: f32, size: Vector2<u32>) -> bool;
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
    size: Vector2<u32>,
    handler: std::rc::Rc<RefCell<dyn WindowHandler + 'static>>,
}


fn create_window(windows: &mut std::collections::HashMap<glutin::window::WindowId, YaffeWindow>,
                 event_loop: &EventLoop<()>, 
                 tracker: &mut context_tracker::ContextTracker, 
                 builder: WindowBuilder,
                 handler: Rc<RefCell<impl WindowHandler + 'static>>) -> Vector2<u32> {
    let windowed_context = glutin::ContextBuilder::new().build_windowed(builder, event_loop).unwrap();
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

    let size = Vector2::new(size.width, size.height);
    let window = YaffeWindow { context_id, renderer, size, handler};
    windows.insert(id, window);
    size
}

pub(crate) fn create_yaffe_windows(notify: std::sync::mpsc::Receiver<u8>,
                                   mut input: impl crate::input::PlatformInput + 'static,
                                   input_map: crate::input::InputMap<VirtualKeyCode, ControllerInput, crate::Actions>,
                                   handler: std::rc::Rc<RefCell<impl WindowHandler + 'static>>,
                                   overlay: std::rc::Rc<RefCell<impl WindowHandler + 'static>>) -> ! {
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
        .with_inner_size(glutin::dpi::PhysicalSize::new(size.x - 1, size.y - 1)) 
        .with_position(glutin::dpi::PhysicalPosition::new(1, 1))
        .with_visible(false)
        .with_always_on_top(true)
        .with_transparent(true)
        .with_decorations(false);
    create_window(&mut windows, &el, &mut ct, builder, overlay);

    for (_, val) in windows.iter_mut() {
        val.handler.borrow_mut().on_start();
    }

    let mut delta_time = 0f32;
    let mut last_time = Instant::now();
    
    el.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Poll;
        
        match event {
            Event::LoopDestroyed => *control_flow = ControlFlow::Exit,

            Event::WindowEvent { event, window_id } => match event {
                WindowEvent::CloseRequested => {
                    let window = windows.get_mut(&window_id).unwrap();
                    *control_flow = ControlFlow::Exit;
                    window.handler.borrow_mut().on_stop();
                }

                WindowEvent::Resized(physical_size) => {
                    let window = windows.get_mut(&window_id).unwrap();
                    let context = ct.get_current(window.context_id).unwrap();

                    let size = Vector2::new(physical_size.width, physical_size.height);
                    window.size = size;
                    context.windowed().resize(physical_size);
                    window.renderer.set_viewport_size_pixels(size);
                    window.handler.borrow_mut().on_resize(physical_size.width, physical_size.height);
                },

                _ => {}
            },
            Event::RedrawRequested(id) => {
                let window = windows.get_mut(&id).unwrap();
                let context = ct.get_current(window.context_id).unwrap();

                let size = window.size;
                let mut handle = window.handler.borrow_mut();
                window.renderer.draw_frame(|graphics| {
                    if !handle.on_frame(graphics, delta_time, size) {
                        *control_flow = ControlFlow::Exit;
                    }
                });
                context.windowed().swap_buffers().unwrap();
            },

            Event::MainEventsCleared => {
                let now = Instant::now();
                delta_time = (now - last_time).as_millis() as f32 / 1000.;
                last_time = now;

                //Get input
                if let Err(e) = input.update(0) {
                    crate::logger::log_entry_with_message(crate::logger::LogTypes::Error, e, "Unable to get input");
                }
                
                //Convert our input to actions we will propogate through the UI
                let mut actions = input_to_action(&input_map, input.get_keyboard(), input.get_gamepad());
                let asset_loaded = notify.try_recv().is_ok();

                for (_, val) in windows.iter_mut() {
                    let mut helper = WindowHelper { visible: None, file_action: None };
                    let context = ct.get_current(val.context_id).unwrap();
                    let mut handle = val.handler.borrow_mut();

                    //Send an action, if its handled remove it so a different window doesnt respond to it
                    let mut handled_actions = Vec::with_capacity(actions.len());
                    for action in actions.iter() {
                        if handle.on_input(&mut helper, action) {
                            handled_actions.push(action.clone());

                            //If the window responded to the action, set it to redraw
                            context.windowed().window().request_redraw();
                        }
                    }
                    for action in handled_actions {
                        actions.remove(&action);
                    }

                    //Method that is always called so we can perform actions always
                    handle.on_fixed_update(&mut helper);
                    helper.resolve(&context.windowed().window());

                    if asset_loaded || handle.is_window_dirty() {
                        context.windowed().window().request_redraw();
                    }
                }
            }

            _ => {}
        }
    });
}

fn input_to_action(input_map: &crate::input::InputMap<VirtualKeyCode, ControllerInput, crate::Actions>, 
                   keyboard: Vec<(VirtualKeyCode, char)>, 
                   gamepad: Vec<ControllerInput>) -> std::collections::HashSet<crate::Actions> {

    let mut result = std::collections::HashSet::new();
    for k in keyboard {
        if let Some(action) = input_map.get(Some(k.0), None) {
            result.insert(*action);
        } else {
            result.insert(crate::Actions::KeyPress(k.1));
        }
    }

    for g in gamepad {
        if let Some(action) = input_map.get(None, Some(g)) {
            result.insert(*action);
        } else {
            result.insert(crate::Actions::KeyPress(g as u8 as char));
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
            FW: FnOnce(WindowedContext<T>, )
                -> Result<WindowedContext<T2>, (WindowedContext<T>, ContextError)>,
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
            ) -> Result<
                ContextWrapper<NotCurrent>,
                (ContextWrapper<PossiblyCurrent>, ContextError),
            >,
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
            ) -> Result<
                ContextWrapper<PossiblyCurrent>,
                (ContextWrapper<NotCurrent>, ContextError),
            >,
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
                                ctx.map(
                                    |ctx| Ok(ctx.treat_as_not_current()),
                                )
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
            F: FnOnce(
                ContextCurrentWrapper,
            )
                -> Result<ContextCurrentWrapper, (ContextCurrentWrapper, ContextError)>,
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
                                    ctx.map(
                                        |ctx| ctx.make_not_current(),
                                    )
                                })
                            }) {
                                panic!(
                                    "Could not `make_current` nor `make_not_current`, {:?}, {:?}",
                                    err, err2
                                );
                            }
                        }

                        if let Err(err2) = self.modify(id, |ctx| {
                            ctx.map_possibly(|ctx| {
                                ctx.map(|ctx| ctx.make_not_current())
                            })
                        }) {
                            panic!(
                                "Could not `make_current` nor `make_not_current`, {:?}, {:?}",
                                err, err2
                            );
                        }

                        return Err(err);
                    }

                    self.current = Some(id);

                    if let Some(old_current) = old_current {
                        self.modify(old_current, |ctx| {
                            ctx.map_possibly(|ctx| {
                                ctx.map(
                                    |ctx| Ok(ctx.treat_as_not_current()),
                                )
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