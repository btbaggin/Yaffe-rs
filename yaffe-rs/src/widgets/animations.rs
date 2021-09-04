use crate::V2;
use crate::widgets::WidgetId;

pub type FieldOffset = usize;

pub trait Animator {
    fn slerp(&self, from: Self, to: Self, amount: f32) -> Self;
}

pub enum AnimationTarget {
    F32((f32, f32)),
    #[allow(dead_code)]
    V2((V2, V2)),
}

pub struct Animation<> {
    widget: WidgetId,
    duration: f32,
    remaining: f32,
    offset: FieldOffset,
    target: AnimationTarget,
}


impl Animator for V2 {
    fn slerp(&self, from: Self, to: Self, amount: f32) -> Self {
        V2::new(self.x.slerp(from.x, to.x, amount), self.y.slerp(from.y, to.y, amount))
    }
}

impl Animator for f32 {
    fn slerp(&self, from: Self, to: Self, amount: f32) -> Self {
        let delta = to - from;
        self + delta * amount
    }
}

impl crate::DeferredAction {
    #[allow(dead_code)]
    pub fn animate_v2(&mut self, widget: &impl crate::widgets::FocusableWidget, field: FieldOffset, target: V2, duration: f32) {
        let anim = Animation {
            widget: widget.get_id(),
            duration: duration,
            remaining: duration,
            offset: field,
            target: AnimationTarget::V2((*apply(field, widget), target)),
        };
        self.anims.push(anim);
    }

    pub fn animate_f32(&mut self, widget: &impl crate::widgets::FocusableWidget, field: FieldOffset, target: f32, duration: f32) {
        let anim = Animation {
            widget: widget.get_id(),
            duration: duration,
            remaining: duration,
            offset: field,
            target: AnimationTarget::F32((*apply(field, widget), target)),
        };
        self.anims.push(anim);
    }
}

/// Processes any widgets that have running animations
/// Currently only position animations are allowed
pub fn run_animations(tree: &mut crate::widgets::WidgetTree, delta_time: f32) {
    //We do this at the beginning because we need animations to persist 1 fram longer than they go
    //This is because we only redraw the screen if animations are playing
    //If we removed them at the end we wouldn't redraw the last frame of the animation
    tree.anims.retain(|a| a.remaining > 0.); 

    //Run animations, if it completes, mark it for removal
    for animation in tree.anims.iter_mut() {
        animation.remaining -= delta_time;
    
        if let Some(widget) = tree.root.find_widget_mut(animation.widget) {

            let widget = widget.widget.as_mut();
            match animation.target {
                AnimationTarget::V2((from, to)) => {
                    let animator = apply_mut::<dyn crate::widgets::Widget, V2>(animation.offset, widget);
                    *animator = animator.slerp(from, to, delta_time / animation.duration);
                    if animation.remaining <= 0. { *animator = to }
                }
                AnimationTarget::F32((from, to)) => {
                    let animator = apply_mut::<dyn crate::widgets::Widget, f32>(animation.offset, widget);
                    *animator = animator.slerp(from, to, delta_time / animation.duration);
                    if animation.remaining <= 0. { *animator = to }
                }
            }
        }
    }
}

#[inline]
fn apply_mut<'a, T: ?Sized, U>(offset: FieldOffset, x: &'a mut T) -> &'a mut U {
    unsafe { &mut *apply_ptr_mut(offset, x) }
}

#[inline]
fn apply_ptr_mut<T: ?Sized, U>(offset: FieldOffset, x: *mut T) -> *mut U {
    ((x as *const() as usize) + offset) as *mut U
}

#[inline]
fn apply_ptr<T, U>(offset: FieldOffset, x: *const T) -> *const U {
    ((x as usize) + offset) as *const U
}

#[inline]
fn apply<'a, T, U>(offset: FieldOffset, x: &'a T) -> &'a U {
    unsafe { &*apply_ptr(offset, x) }
}

#[macro_export]
macro_rules! offset_of {
    ($t: path => $f: tt) => {{
        // Construct the offset
        #[allow(unused_unsafe)]
        unsafe {
            let uninit = std::mem::MaybeUninit::<$t>::uninit();
            let base_ptr = uninit.as_ptr();
            let field_ptr = memoffset::raw_field!(base_ptr, $t, $f);
            (field_ptr as usize).wrapping_sub(base_ptr as usize) as crate::widgets::animations::FieldOffset
        }
    }};
    ($t: path => $f: ident: $($rest: tt)*) => {
        $crate::offset_of!($t => $f) + $crate::offset_of!($($rest)*)
    };
}