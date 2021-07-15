use speedy2d::*;
use speedy2d::dimen::Vector2;
use glutin::event::{Event, WindowEvent, VirtualKeyCode};
use glutin::event_loop::{ControlFlow, EventLoop};
use glutin::window::{Fullscreen, WindowBuilder};
use crate::{V2, ControllerInput};

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

enum WindowVisibility {
    Visible,
    Hide,
    None,
}

pub struct WindowHelper {
    visible: WindowVisibility,
}

impl WindowHelper {
    pub fn set_visibility(&mut self, visible: bool) {
        if visible { self.visible = WindowVisibility::Visible; }
        else { self.visible = WindowVisibility::Hide; }
    }
}

pub(crate) trait WindowHandler {
    fn on_start(&mut self) { }
    fn on_frame_start(&mut self, _: &mut WindowHelper) { }
    fn on_frame(&mut self, graphics: &mut Graphics2D, size: Vector2<u32>) -> bool;
    fn on_input(&mut self, helper: &mut WindowHelper, key: Option<VirtualKeyCode>, button: Option<ControllerInput>) -> bool;
    fn on_resize(&mut self, _: u32, _: u32) { }
    fn on_stop(&mut self) { }
}

struct YaffeWindow {
    context_id: usize,
    renderer: GLRenderer,
    size: Vector2<u32>,
    handler: std::rc::Rc<RefCell<dyn WindowHandler + 'static>>,
}

use std::rc::Rc;
use std::cell::RefCell;
fn create_window(windows: &mut std::collections::HashMap<glutin::window::WindowId, YaffeWindow>,
                 event_loop: &EventLoop<()>, 
                 tracker: &mut context_tracker::ContextTracker, 
                 title: &str,
                 visible: bool,
                 transparent: bool,
                 handler: Rc<RefCell<impl WindowHandler + 'static>>) {
    let fullscreen = Some(Fullscreen::Borderless(event_loop.primary_monitor()));
    let builder = WindowBuilder::new()
        .with_title(title)  
        .with_fullscreen(fullscreen.clone())
        .with_visible(visible)
        .with_transparent(transparent)
        .with_always_on_top(transparent);//TODO look at this

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

    let window = YaffeWindow { context_id, renderer, size: Vector2::new(size.width, size.height), handler: handler};
    windows.insert(id, window);
}

pub(crate) fn create_yaffe_windows(handler: std::rc::Rc<RefCell<impl WindowHandler + 'static>>,
                                   overlay: std::rc::Rc<RefCell<impl WindowHandler + 'static>>) -> ! {
    let el = EventLoop::new();
    
    let mut ct = context_tracker::ContextTracker::default();
    let mut windows = std::collections::HashMap::new();

    // //https://github.com/rust-windowing/glutin/blob/master/glutin_examples/examples/multiwindow.rs
    create_window(&mut windows, &el, &mut ct, "Yaffe", true, false, handler);
    create_window(&mut windows, &el, &mut ct, "Overlay", false, true, overlay);

    for (_, val) in windows.iter_mut() {
        val.handler.borrow_mut().on_start();
    }
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
                //TODO frame limit
                let window = windows.get_mut(&id).unwrap();
                let context = ct.get_current(window.context_id).unwrap();
                let size = window.size;
                let mut handle = window.handler.borrow_mut();
                window.renderer.draw_frame(|graphics| {
                    if !handle.on_frame(graphics, size) {
                        *control_flow = ControlFlow::Exit;
                    }
                });
                context.windowed().swap_buffers().unwrap();
            },

            Event::RedrawEventsCleared => {
                //TODO only request if animations are playing?
                for (_, val) in windows.iter_mut() {
                    let context = ct.get_current(val.context_id).unwrap();
                    context.windowed().window().request_redraw();
                }
            }

            Event::MainEventsCleared => {
                //TODO need to take out keys if on_input returns true
                let (keyboard, gamepad) = crate::platform_layer::get_input();
                for (_, val) in windows.iter_mut() {
                    let mut helper = WindowHelper { visible: WindowVisibility::None };
                    let context = ct.get_current(val.context_id).unwrap();
                    let mut handle = val.handler.borrow_mut();

                    for k in keyboard.iter() { handle.on_input(&mut helper, Some(*k), None); }
                    for g in gamepad.iter() { handle.on_input(&mut helper, None, Some(*g)); }

                    handle.on_frame_start(&mut helper);
                    resolve_window_helper(helper, &context.windowed().window());
                }
            }

            _ => {}
        }
    });
}

fn resolve_window_helper(helper: WindowHelper, window: &glutin::window::Window) {
    match helper.visible {
        WindowVisibility::Hide => {
            window.set_visible(false);
        }
        WindowVisibility::Visible => {
            window.set_visible(true);
        }
        WindowVisibility::None => { }
    }
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