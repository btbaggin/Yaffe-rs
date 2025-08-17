use crate::{Actions, Graphics, LogicalPosition, LogicalSize, Rect};

mod animations;
mod controls;
mod deferred_action;
mod modal;
mod utils;
mod widget_tree;
pub use animations::{AnimationManager, FieldOffset};
pub use controls::*;
pub use deferred_action::DeferredAction;
pub use modal::*;
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
    ($vis:vis struct $name:ident {
        $($element:ident: $ty:ty = $value:expr),*
    }) => {
        #[allow(unused_variables)]
        $vis struct $name {
            position: $crate::LogicalPosition,
            size: $crate::LogicalSize,
            id: $crate::ui::WidgetId,
            $($element: $ty),*
        }
        impl $crate::ui::LayoutElement for $name {
            fn layout(&self) -> $crate::Rect { $crate::Rect::new(self.position, self.position + self.size) }
            fn set_layout(&mut self, layout: $crate::Rect) {
                self.position = *layout.top_left();
                self.size = layout.size();
            }
            fn get_id(&self) -> WidgetId { self.id }
            fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
        }
        impl $name {
            #[allow(dead_code)]
            pub fn new_with_id(id: WidgetId) -> $name {
                $name {
                    position: $crate::LogicalPosition::new(0., 0.),
                    size: $crate::LogicalSize::new(0., 0.),
                    id,
                    $($element: $value),*
                }
            }

            #[allow(dead_code)]
            pub fn new() -> $name {
                Self::new_with_id(WidgetId::random())
            }
        }
    };
}

pub struct ContainerChild<T, D> {
    element: Box<dyn UiElement<T, D>>,
    size: ContainerSize,
}
impl<T, D> ContainerChild<T, D> {
    fn get_size(&self, total: f32, fill_size: f32) -> f32 {
        match self.size {
            ContainerSize::Percent(p) => total * p,
            ContainerSize::Fixed(f) => f,
            ContainerSize::Fill => fill_size,
        }
    }
}

pub enum ContainerSize {
    Percent(f32),
    Fixed(f32),
    Fill,
}

enum FlexDirection {
    Row,
    Column,
}

pub struct UiContainer<T: 'static, D: 'static> {
    position: LogicalPosition,
    size: LogicalSize,
    id: WidgetId,
    children: Vec<ContainerChild<T, D>>,
    direction: FlexDirection,
    fill_count: usize,
}
impl<T, D> LayoutElement for UiContainer<T, D> {
    fn layout(&self) -> Rect { Rect::new(self.position, self.position + self.size) }
    fn set_layout(&mut self, layout: Rect) {
        self.position = *layout.top_left();
        self.size = layout.size();
    }
    fn get_id(&self) -> WidgetId { self.id }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
impl<T, D> UiContainer<T, D> {
    pub fn row() -> UiContainer<T, D> {
        UiContainer {
            position: LogicalPosition::new(0., 0.),
            size: LogicalSize::new(0., 0.),
            id: WidgetId::random(),
            children: vec![],
            direction: FlexDirection::Row,
            fill_count: 0,
        }
    }

    pub fn column() -> UiContainer<T, D> {
        UiContainer {
            position: LogicalPosition::new(0., 0.),
            size: LogicalSize::new(0., 0.),
            id: WidgetId::random(),
            children: vec![],
            direction: FlexDirection::Column,
            fill_count: 0,
        }
    }

    pub fn add_child(&mut self, child: impl UiElement<T, D> + 'static, size: ContainerSize) -> &mut Self {
        if let ContainerSize::Fill = size {
            self.fill_count += 1;
        }

        let child = ContainerChild { element: Box::new(child), size };
        self.children.push(child);
        self
    }

    pub fn with_child(&mut self, child: UiContainer<T, D>, size: ContainerSize) -> &mut UiContainer<T, D> {
        self.add_child(child, size);

        let count = self.children.len();
        self.children[count - 1].element.as_mut().as_any_mut().downcast_mut::<UiContainer<T, D>>().unwrap()
    }

    pub fn find_widget_mut(&mut self, widget_id: WidgetId) -> Option<&mut dyn UiElement<T, D>> {
        // Check if the current container matches the widget_id
        if self.get_id() == widget_id {
            return Some(self);
        }

        // Recursively search in children
        for child in &mut self.children {
            if child.element.get_id() == widget_id {
                return Some(child.element.as_mut());
            } else if let Some(container) = child.element.as_any_mut().downcast_mut::<UiContainer<T, D>>() {
                if let Some(found) = container.find_widget_mut(widget_id) {
                    return Some(found);
                }
            }
        }

        None
    }

    fn calc_fill_size(&self, total: f32) -> f32 {
        if self.fill_count == 0 {
            return 0.;
        }

        let mut total = total;
        for child in &self.children {
            total -= child.get_size(total, 0.);
        }
        total / self.fill_count as f32
    }
}

impl<T: 'static, D: 'static> UiElement<T, D> for UiContainer<T, D> {
    fn render(&mut self, graphics: &mut Graphics, state: &T, current_focus: &WidgetId) {
        let total = match self.direction {
            FlexDirection::Row => graphics.bounds.width(),
            FlexDirection::Column => graphics.bounds.height(),
        };

        let fill_size = self.calc_fill_size(total);

        for child in &mut self.children {
            let (width, height, x_offset, y_offset) = match self.direction {
                FlexDirection::Row => {
                    let width = child.get_size(total, fill_size);
                    let height = graphics.bounds.height();
                    (width, height, width, 0.)
                }
                FlexDirection::Column => {
                    let width = graphics.bounds.width();
                    let height = child.get_size(total, fill_size);
                    (width, height, 0., height)
                }
            };

            let origin = *graphics.bounds.top_left();
            let size = graphics.bounds.size();
            child.element.set_layout(Rect::point_and_size(origin, LogicalSize::new(width, height)));

            // graphics.bounds = Rect::point_and_size(origin, LogicalSize::new(width, height));
            child.element.render(graphics, state, current_focus);
            graphics.bounds = Rect::point_and_size(
                origin + LogicalPosition::new(x_offset, y_offset),
                LogicalSize::new(size.x - x_offset, size.y - y_offset),
            );
        }
    }

    fn action(&mut self, state: &mut T, animations: &mut AnimationManager, action: &Actions, handler: &mut D) -> bool {
        // TODO only do current focus
        for child in &mut self.children {
            if child.element.action(state, animations, action, handler) {
                return true;
            }
        }
        false
    }
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
