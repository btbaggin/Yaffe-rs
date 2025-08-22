use crate::ui::WidgetId;

pub type FieldOffset = usize;

pub enum AnimationData {
    F32 { from: f32, to: f32 },
}

pub trait Animator: Clone + 'static {
    fn to_animation_data(from: Self, to: Self) -> AnimationData;
    fn slerp(&self, from: Self, to: Self, amount: f32) -> Self;
}
impl Animator for f32 {
    fn to_animation_data(from: Self, to: Self) -> AnimationData { AnimationData::F32 { from, to } }

    fn slerp(&self, from: Self, to: Self, amount: f32) -> Self {
        let delta = to - from;
        self + delta * amount
    }
}

struct Animation {
    widget: WidgetId,
    duration: f32,
    remaining: f32,
    offset: FieldOffset,
    data: AnimationData,
}

pub struct AnimationBuilder<'a, T: Animator> {
    manager: &'a mut AnimationManager,
    widget_id: WidgetId,
    field: FieldOffset,
    target: T,
    duration: f32,
    current_value: T,
}
impl<'a, T: Animator> AnimationBuilder<'a, T> {
    pub fn duration(mut self, duration: f32) -> Self {
        self.duration = duration;
        self
    }

    pub fn start(self) {
        let data = T::to_animation_data(self.current_value, self.target);
        let anim = Animation {
            widget: self.widget_id,
            duration: self.duration,
            remaining: self.duration,
            offset: self.field,
            data,
        };
        self.manager.animations.push(anim);
    }
}

pub struct AnimationManager {
    animations: Vec<Animation>,
}
impl AnimationManager {
    pub fn new() -> AnimationManager { AnimationManager { animations: vec![] } }

    pub fn is_dirty(&self) -> bool { !self.animations.is_empty() }

    pub fn animate<'a, T: Animator>(
        &'a mut self,
        widget: &impl crate::ui::LayoutElement,
        field: FieldOffset,
        target: T,
    ) -> AnimationBuilder<'a, T> {
        let current_value = apply::<_, T>(field, widget).clone();

        AnimationBuilder {
            manager: self,
            widget_id: widget.get_id(),
            field,
            target,
            current_value,
            duration: 0.3,
        }
    }

    /// Processes any widgets that have running animations
    /// Currently only position animations are allowed
    pub fn process<S: 'static, D: 'static>(&mut self, root: &mut crate::ui::UiContainer<S, D>, delta_time: f32) {
        //We do this at the beginning because we need animations to persist 1 fram longer than they go
        //This is because we only redraw the screen if animations are playing
        //If we removed them at the end we wouldn't redraw the last frame of the animation
        self.animations.retain(|a| a.remaining > 0.);

        //Run animations, if it completes, mark it for removal
        for animation in self.animations.iter_mut() {
            animation.remaining -= delta_time;

            if let Some(widget) = root.find_widget_mut(animation.widget) {
                match animation.data {
                    AnimationData::F32 { from, to } => {
                        let animator = apply_mut::<dyn crate::ui::UiElement<S, D>, f32>(animation.offset, widget);
                        *animator = animator.slerp(from, to, delta_time / animation.duration);
                        if animation.remaining <= 0. {
                            *animator = to
                        }
                    }
                }
            } else {
                animation.remaining = 0.;
            }
        }
    }
}

//Inspired, but greatly simplified from https://github.com/Diggsey/rust-field-offset
#[inline]
fn apply_mut<T: ?Sized, U>(offset: FieldOffset, x: &mut T) -> &mut U { unsafe { &mut *apply_ptr_mut(offset, x) } }

#[inline]
fn apply_ptr_mut<T: ?Sized, U>(offset: FieldOffset, x: *mut T) -> *mut U {
    ((x as *const () as usize) + offset) as *mut U
}

#[inline]
fn apply_ptr<T: ?Sized, U>(offset: FieldOffset, x: *const T) -> *const U {
    ((x as *const () as usize) + offset) as *const U
}

#[inline]
fn apply<T: ?Sized, U>(offset: FieldOffset, x: &T) -> &U { unsafe { &*apply_ptr(offset, x) } }

#[macro_export]
macro_rules! offset_of {
    ($t: path => $f: tt) => {{
        // Construct the offset
        #[allow(unused_unsafe)]
        unsafe {
            let uninit = std::mem::MaybeUninit::<$t>::uninit();
            let base_ptr = uninit.as_ptr();
            let field_ptr = memoffset::raw_field!(base_ptr, $t, $f);
            (field_ptr as usize).wrapping_sub(base_ptr as usize) as $crate::ui::FieldOffset
        }
    }};
    ($t: path => $f: ident: $($rest: tt)*) => {
        $crate::offset_of!($t => $f) + $crate::offset_of!($($rest)*)
    };
}
