use crate::{Actions, Graphics, LogicalSize, Rect};
use speedy2d::color::Color;

mod animations;
mod controls;
mod deferred_action;
mod modal;
mod ui_container;
mod utils;
mod widget_tree;
pub use animations::{AnimationManager, FieldOffset};
pub use controls::*;
pub use deferred_action::DeferredAction;
pub use modal::*;
pub use ui_container::*;
pub use utils::*;
pub use widget_tree::{WidgetTree, WindowState};

#[repr(u8)]
enum FocusType {
    Revert,
    Focus(WidgetId),
}

pub trait LayoutElement {
    fn layout(&self) -> Rect;
    fn set_layout(&mut self, layout: Rect);
    fn get_id(&self) -> WidgetId;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
pub trait UiElement<T: 'static, D: 'static>: LayoutElement {
    fn calc_size(&mut self, _: &mut Graphics) -> LogicalSize { LogicalSize::new(0., 0.) }
    fn render(&mut self, graphics: &mut Graphics, state: &T, current_focus: &WidgetId);
    fn action(&mut self, _state: &mut T, _: &mut AnimationManager, _: &Actions, _handler: &mut D) -> bool { false }
    fn got_focus(&mut self, _: &T, _: &mut AnimationManager) {}
    fn lost_focus(&mut self, _: &T, _: &mut AnimationManager) {}
}
pub trait ValueElement<T> {
    fn value(&self) -> T;
}

#[macro_export]
macro_rules! convert_to {
    ($element:expr, $ty:ty) => {
        $element.as_any().downcast_ref::<$ty>().unwrap()
    };
}

#[derive(Copy, Clone, PartialEq)]
pub struct WidgetId(u64);
impl WidgetId {
    pub const fn static_id(id: u64) -> Self { WidgetId(id) }
    pub fn random() -> Self {
        use rand::Rng;
        WidgetId(rand::rng().random::<u64>())
    }
}

#[macro_export]
macro_rules! widget {
    ($vis:vis struct $name:ident $(<$($generic:ident $( : $bound:path )?),+>)? {
        $($elevis:vis $element:ident: $ty:ty = $value:expr),*
    }) => {
        #[allow(unused_variables)]
        $vis struct $name $(<$($generic : 'static $(+ $bound)?),+>)? {
            position: $crate::LogicalPosition,
            size: $crate::LogicalSize,
            id: $crate::ui::WidgetId,
            $($elevis $element: $ty),*
        }
        impl $(<$($generic : 'static $(+ $bound)?),+>)? $crate::ui::LayoutElement for $name $(<$($generic),+>)? {
            fn layout(&self) -> $crate::Rect { $crate::Rect::new(self.position, self.position + self.size) }
            fn set_layout(&mut self, layout: $crate::Rect) {
                self.position = *layout.top_left();
                self.size = layout.size();
            }
            fn get_id(&self) -> $crate::ui::WidgetId { self.id }
            fn as_any(&self) -> &dyn std::any::Any { self }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
        }
        impl $(<$($generic : 'static $(+ $bound)?),+>)? $name $(<$($generic),+>)? {
            #[allow(dead_code)]
            pub fn new_with_id(id: $crate::ui::WidgetId) -> $name $(<$($generic),+>)? {
                $name {
                    position: $crate::LogicalPosition::new(0., 0.),
                    size: $crate::LogicalSize::new(0., 0.),
                    id,
                    $($element: $value),*
                }
            }

            #[allow(dead_code)]
            pub fn new() -> $name $(<$($generic),+>)? {
                Self::new_with_id($crate::ui::WidgetId::random())
            }
        }
    };
}
