use crate::ui::WidgetId;

pub type FieldOffset = usize;

pub trait Animator {
    fn slerp(&self, from: Self, to: Self, amount: f32) -> Self;
}
impl Animator for f32 {
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

enum AnimationData {
    F32 { from: f32, to: f32 },
}

pub struct AnimationManager {
    animations: Vec<Animation>,
}
impl AnimationManager {
    pub fn new() -> AnimationManager { AnimationManager { animations: vec![] } }

    pub fn is_dirty(&self) -> bool { !self.animations.is_empty() }

    pub fn animate_f32(
        &mut self,
        widget: &impl crate::ui::FocusableWidget,
        field: FieldOffset,
        target: f32,
        duration: f32,
    ) {
        let data = AnimationData::F32 { from: *apply(field, widget), to: target };

        let anim = Animation { widget: widget.get_id(), duration, remaining: duration, offset: field, data };
        self.animations.push(anim);
    }

    /// Processes any widgets that have running animations
    /// Currently only position animations are allowed
    pub fn process(&mut self, root: &mut crate::ui::WidgetContainer, delta_time: f32) {
        //We do this at the beginning because we need animations to persist 1 fram longer than they go
        //This is because we only redraw the screen if animations are playing
        //If we removed them at the end we wouldn't redraw the last frame of the animation
        self.animations.retain(|a| a.remaining > 0.);

        //Run animations, if it completes, mark it for removal
        for animation in self.animations.iter_mut() {
            animation.remaining -= delta_time;

            if let Some(widget) = root.find_widget_mut(animation.widget) {
                let widget = widget.widget.as_mut();
                match animation.data {
                    AnimationData::F32 { from, to } => {
                        let animator = apply_mut::<dyn crate::ui::Widget, f32>(animation.offset, widget);
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
fn apply_ptr<T, U>(offset: FieldOffset, x: *const T) -> *const U { ((x as usize) + offset) as *const U }

#[inline]
fn apply<T, U>(offset: FieldOffset, x: &T) -> &U { unsafe { &*apply_ptr(offset, x) } }

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
