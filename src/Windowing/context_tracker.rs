use glutin::{self, ContextCurrentState, ContextError, NotCurrent, PossiblyCurrent, WindowedContext};
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

#[allow(clippy::result_large_err)]
fn map<T2: ContextCurrentState, FW>(self, fw: FW) -> Result<ContextWrapper<T2>, (Self, ContextError)>
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

#[allow(clippy::result_large_err)]
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
                        ctx.map_possibly(|ctx| { ctx.map(|ctx| Ok(ctx.treat_as_not_current())) })
                    }).unwrap()
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
                    ctx.map_not(|ctx| { ctx.map(|ctx| ctx.make_current()) })
                }) {
                    // Oh noes, something went wrong
                    // Let's at least make sure that no context is current.
                    if let Some(old_current) = old_current {
                        if let Err(err2) = self.modify(old_current, |ctx| {
                            ctx.map_possibly(|ctx| { ctx.map(|ctx| ctx.make_not_current()) })
                        }) {
                            panic!("Could not `make_current` nor `make_not_current`, {:?}, {:?}", err, err2);
                        }
                    }

                    if let Err(err2) = self.modify(id, |ctx| {
                        ctx.map_possibly(|ctx| { ctx.map(|ctx| ctx.make_not_current()) })
                    }) {
                        panic!("Could not `make_current` nor `make_not_current`, {:?}, {:?}", err, err2);
                    }

                    return Err(err);
                }

                self.current = Some(id);

                if let Some(old_current) = old_current {
                    self.modify(old_current, |ctx| {
                        ctx.map_possibly(|ctx| { ctx.map(|ctx| Ok(ctx.treat_as_not_current())) })
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