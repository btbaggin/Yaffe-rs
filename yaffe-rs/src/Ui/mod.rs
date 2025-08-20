use crate::{Actions, Graphics, Rect};
use speedy2d::color::Color;

mod animations;
mod controls;
mod deferred_action;
mod modal;
mod utils;
mod widget_tree;
mod ui_container;
pub use animations::{AnimationManager, FieldOffset};
pub use controls::*;
pub use deferred_action::DeferredAction;
pub use modal::*;
pub use utils::*;
pub use widget_tree::{WidgetTree, WindowState};
pub use ui_container::*;

#[repr(u8)]
enum FocusType {
    Revert,
    Focus(WidgetId),
}

pub trait LayoutElement {
    fn id(&self) -> WidgetId;
    fn layout(&self) -> Rect;
    fn set_layout(&mut self, layout: Rect);
    fn get_id(&self) -> WidgetId;
    fn as_any(&self) -> &dyn std::any::Any;
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any;
}
pub trait UiElement<T: 'static, D: 'static>: LayoutElement {
    fn render(&mut self, graphics: &mut Graphics, state: &T, current_focus: &WidgetId);
    fn action(&mut self, _state: &mut T, _: &mut AnimationManager, _: &Actions, _handler: &mut D) -> bool {
        false
    }
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
        impl  $(<$($generic : 'static $(+ $bound)?),+>)? $crate::ui::LayoutElement for $name $(<$($generic),+>)? {
            fn id(&self) -> $crate::ui::WidgetId { self.id }
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



// /// Contains an individual widget
// /// Manages ratio sizing and animations
// /// T is the global state that is persisted across the entire application
// /// D is the per interaction state thats passed down for each interaction
// pub struct WidgetContainer<T, D> {
//     children: Vec<WidgetContainer<T, D>>,
//     widget: Box<dyn Widget<T, D>>,
//     ratio: LogicalSize,
//     alignment: ContainerAlignment,
// }
// impl<T, D> WidgetContainer<T, D> {
//     pub fn root(widget: impl Widget<T, D> + 'static) -> WidgetContainer<T, D> {
//         WidgetContainer::new(widget, LogicalSize::new(1.0, 1.0), ContainerAlignment::Left)
//     }
//     fn new(
//         widget: impl Widget<T, D> + 'static,
//         ratio: LogicalSize,
//         alignment: ContainerAlignment,
//     ) -> WidgetContainer<T, D> {
//         WidgetContainer { children: vec![], widget: Box::new(widget), ratio, alignment }
//     }

//     pub fn add_child(
//         &mut self,
//         widget: impl Widget<T, D> + 'static,
//         size: LogicalSize,
//         alignment: ContainerAlignment,
//     ) -> &mut Self {
//         self.children.push(WidgetContainer::new(widget, size, alignment));
//         self
//     }

//     pub fn with_child(&mut self, widget: impl Widget<T, D> + 'static, size: LogicalSize) -> &mut WidgetContainer<T, D> {
//         self.children.push(WidgetContainer::new(widget, size, ContainerAlignment::Left));

//         let count = self.children.len();
//         &mut self.children[count - 1]
//     }

//     pub fn action(
//         &mut self,
//         state: &mut T,
//         animations: &mut AnimationManager,
//         action: &Actions,
//         current_focus: &WidgetId,
//         handler: &mut D,
//     ) -> bool {
//         //Only send action to currently focused widget
//         let handled = current_focus == &self.widget.get_id() && self.widget.action(state, animations, action, handler);
//         if handled {
//             return true;
//         }

//         for i in self.children.iter_mut() {
//             if i.action(state, animations, action, current_focus, handler) {
//                 return true;
//             }
//         }

//         false
//     }

//     pub fn render(&mut self, state: &T, graphics: &mut Graphics, current_focus: &WidgetId) {
//         //These measure the offset from the edges to begin or end rendering
//         let mut top_stack = 0.;
//         let mut bottom_stack = 0.;
//         let mut left_stack = 0.;
//         let mut right_stack = 0.;

//         let rect = self.widget.layout();
//         graphics.bounds = rect;
//         self.widget.render(graphics, state, current_focus);

//         for i in self.children.iter_mut() {
//             let size = LogicalSize::new(rect.width() * i.ratio.x, rect.height() * i.ratio.y);

//             let origin;
//             match i.alignment {
//                 ContainerAlignment::Left => {
//                     origin = LogicalPosition::new(rect.left() + left_stack, rect.top());
//                     left_stack += size.x;
//                 }
//                 ContainerAlignment::Right => {
//                     origin = LogicalPosition::new(rect.right() - (size.x + right_stack), rect.top());
//                     right_stack += size.x;
//                 }
//                 ContainerAlignment::Top => {
//                     origin = LogicalPosition::new(rect.left(), rect.top() + top_stack);
//                     top_stack += size.y;
//                 }
//                 ContainerAlignment::Bottom => {
//                     origin = LogicalPosition::new(rect.left(), rect.bottom() - (size.y + bottom_stack));
//                     bottom_stack += size.y;
//                 }
//             };

//             let offset = i.widget.offset();
//             let origin = LogicalPosition::new(origin.x + offset.x * size.x, origin.y + offset.y * size.y);
//             let r = Rect::new(origin, origin + size);
//             i.widget.set_layout(r);

//             i.render(state, graphics, current_focus);
//         }
//     }

//     fn find_widget_mut(&mut self, widget: WidgetId) -> Option<&mut WidgetContainer<T, D>> {
//         let id = self.widget.get_id();
//         if widget == id {
//             return Some(self);
//         }

//         for i in self.children.iter_mut() {
//             let widget = i.find_widget_mut(widget);
//             if widget.is_some() {
//                 return widget;
//             }
//         }
//         None
//     }
// }
