use crate::{YaffeState, Graphics, Actions, LogicalPosition, LogicalSize, Rect};

mod animations;
mod utils;
mod deferred_action;
mod widget_tree;
mod controls;
mod modal;
pub use animations::{AnimationManager, FieldOffset, AnimationTarget};
pub use widget_tree::{WidgetTree, ContainerAlignment};
pub use deferred_action::DeferredAction;
pub use utils::*;
pub use controls::*;
pub use modal::*;

#[repr(u8)]
enum FocusType {
    Revert,
    Focus(WidgetId),
}


pub trait UiElement {
    fn position(&self) -> LogicalPosition;
    fn size(&self) -> LogicalSize;
    fn layout(&self) -> Rect;
    fn set_layout(&mut self, layout: Rect);
}
pub type WidgetId = std::any::TypeId;
pub trait FocusableWidget: UiElement {
    fn get_id(&self) -> WidgetId;
}
pub trait Widget: FocusableWidget {
    /// Update and draw
    fn render(&mut self, graphics: &mut Graphics, state: &YaffeState);

    /// Offset from initial placement to move. Percentage based on widget size
    fn offset(&self) -> LogicalPosition { LogicalPosition::new(0., 0.) }

    /// Called when a user action occurs
    fn action(&mut self, _: &mut YaffeState, _: &Actions, _: &mut DeferredAction) -> bool { false }

    /// Called when the control gets focus
    fn got_focus(&mut self, _: &YaffeState) {}

    /// Called when the control loses focus
    fn lost_focus(&mut self, _: &YaffeState) {}
}

#[macro_export]
macro_rules! get_widget_id {
    ($widget:ty) => {
        std::any::TypeId::of::<$widget>()
    };
}

#[macro_export]
macro_rules! widget {
    (pub struct $name:ident {
        $($element:ident: $ty:ty = $value:expr),*
    }) => {
        #[allow(unused_variables)]
        pub struct $name { 
            position: $crate::LogicalPosition,
            size: $crate::LogicalSize,
            animator: std::rc::Rc<std::cell::RefCell<$crate::ui::AnimationManager>>,
            $($element: $ty),* 
        }
        impl $crate::ui::UiElement for $name {
            fn position(&self) -> $crate::LogicalPosition { self.position }
            fn size(&self) -> $crate::LogicalSize { self.size }
            fn layout(&self) -> $crate::Rect { $crate::Rect::new(self.position, self.position + self.size) }
            fn set_layout(&mut self, layout: $crate::Rect) { 
                self.position = *layout.top_left(); 
                self.size = layout.size();
            }
        }
        impl $crate::ui::FocusableWidget for $name {
            fn get_id(&self) -> $crate::ui::WidgetId { std::any::TypeId::of::<$name>() }
        }
        impl $name {
            pub fn new(animator: std::rc::Rc<std::cell::RefCell<$crate::ui::AnimationManager>>) -> $name {
                $name { 
                    position: $crate::LogicalPosition::new(0., 0.),
                    size: $crate::LogicalSize::new(0., 0.),
                    animator,
                    $($element: $value),*
                }
            }

            #[allow(dead_code)]
            pub fn animate(&mut self, field: $crate::ui::FieldOffset, target: f32, duration: f32) {
                use $crate::ui::AnimationTarget;

                let mut animator = self.animator.borrow_mut();
                animator.animate(self, field, AnimationTarget::F32(target), duration);
            }
        }
    };
}

/// Contains an individual widget
/// Manages ratio sizing and animations
pub struct WidgetContainer {
    children: Vec<WidgetContainer>,
    widget: Box<dyn Widget>,
    ratio: LogicalSize,
    alignment: ContainerAlignment,
}
impl WidgetContainer {
    pub fn root(widget: impl Widget + 'static) -> WidgetContainer {
        WidgetContainer::new(widget, LogicalSize::new(1.0, 1.0), ContainerAlignment::Left)
    }
    fn new(widget: impl Widget + 'static, ratio: LogicalSize, alignment: ContainerAlignment) -> WidgetContainer {
         WidgetContainer {
            children: vec!(),
            widget: Box::new(widget),
            ratio,
            alignment,
         }
    }

    pub fn add_child(&mut self, widget: impl Widget + 'static, size: LogicalSize, alignment: ContainerAlignment) -> &mut Self {
        self.children.push(WidgetContainer::new(widget, size, alignment));
        self
    }

    pub fn with_child(&mut self, widget: impl Widget + 'static, size: LogicalSize) -> &mut WidgetContainer {
        self.children.push(WidgetContainer::new(widget, size, ContainerAlignment::Left));

        let count = self.children.len();
        &mut self.children[count - 1]
    }

    pub fn action(&mut self, state: &mut YaffeState, action: &Actions, current_focus: &WidgetId, handler: &mut DeferredAction) -> bool {
        //Only send action to currently focused widget
        let handled = current_focus == &self.widget.get_id() && self.widget.action(state, action, handler);
        if handled { return true; }

        for i in self.children.iter_mut() {
            if i.action(state, action, current_focus, handler) { return true; }
        }

        false
    }

    pub fn render(&mut self, state: &YaffeState, graphics: &mut Graphics) {
        //These measure the offset from the edges to begin or end rendering
        let mut top_stack = 0.;
        let mut bottom_stack = 0.;
        let mut left_stack = 0.;
        let mut right_stack = 0.;

        let rect = self.widget.layout();
        graphics.bounds = rect;
        self.widget.render(graphics, state);

        for i in self.children.iter_mut() {
            let size = LogicalSize::new(rect.width() * i.ratio.x, rect.height() * i.ratio.y);

            let origin;
            match i.alignment {
                ContainerAlignment::Left => {
                    origin = LogicalPosition::new(rect.left() + left_stack, rect.top());
                    left_stack += size.x;
                },
                ContainerAlignment::Right => {
                    origin = LogicalPosition::new(rect.right() - (size.x + right_stack), rect.top());
                    right_stack += size.x;
                },
                ContainerAlignment::Top => {
                    origin = LogicalPosition::new(rect.left(), rect.top() + top_stack);
                    top_stack += size.y;
                },
                ContainerAlignment::Bottom => {
                    origin = LogicalPosition::new(rect.left(), rect.bottom() - (size.y + bottom_stack));
                    bottom_stack += size.y;
                },
            };

            let offset = i.widget.offset();
            let origin = LogicalPosition::new(origin.x + offset.x * size.x, origin.y + offset.y * size.y);
            let r = Rect::new(origin, origin + size);
            i.widget.set_layout(r);

            i.render(state, graphics);
        }
    }

    fn find_widget_mut(&mut self, widget: WidgetId) -> Option<&mut WidgetContainer> {
        let id = self.widget.get_id();
        if widget == id { return Some(self) }

        for i in self.children.iter_mut() {
            let widget = i.find_widget_mut(widget);
            if widget.is_some() { return widget; }
        }
        None
    }
}
