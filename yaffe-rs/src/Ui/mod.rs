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
pub use widget_tree::{ContainerAlignment, WidgetTree, WindowState};

#[repr(u8)]
enum FocusType {
    Revert,
    Focus(WidgetId),
}

pub trait UiElement {
    fn layout(&self) -> Rect;
    fn set_layout(&mut self, layout: Rect);
}
#[derive(Copy, Clone, PartialEq)]
pub struct WidgetId(std::any::TypeId);
impl WidgetId {
    pub fn of<T: 'static>() -> WidgetId { WidgetId(std::any::TypeId::of::<T>()) }
    pub fn is_focused<T: 'static>(&self) -> bool { self == &WidgetId::of::<T>() }
}

pub trait FocusableWidget: UiElement {
    fn get_id(&self) -> WidgetId;
}
pub trait Widget<T, D>: FocusableWidget {
    /// Update and draw
    fn render(&mut self, graphics: &mut Graphics, state: &T, current_focus: &WidgetId);

    /// Offset from initial placement to move. Percentage based on widget size
    fn offset(&self) -> LogicalPosition { LogicalPosition::new(0., 0.) }

    /// Called when a user action occurs
    fn action(&mut self, _: &mut T, _: &mut AnimationManager, _: &Actions, _: &mut D) -> bool { false }

    /// Called when the control gets focus
    fn got_focus(&mut self, _: &T, _: &mut AnimationManager) {}

    /// Called when the control loses focus
    fn lost_focus(&mut self, _: &T, _: &mut AnimationManager) {}
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
            $($element: $ty),*
        }
        impl $crate::ui::UiElement for $name {
            fn layout(&self) -> $crate::Rect { $crate::Rect::new(self.position, self.position + self.size) }
            fn set_layout(&mut self, layout: $crate::Rect) {
                self.position = *layout.top_left();
                self.size = layout.size();
            }
        }
        impl $crate::ui::FocusableWidget for $name {
            fn get_id(&self) -> $crate::ui::WidgetId { $crate::ui::WidgetId::of::<$name>() }
        }
        impl $name {
            pub fn new() -> $name {
                $name {
                    position: $crate::LogicalPosition::new(0., 0.),
                    size: $crate::LogicalSize::new(0., 0.),
                    $($element: $value),*
                }
            }
        }
    };
}

/// Contains an individual widget
/// Manages ratio sizing and animations
/// T is the global state that is persisted across the entire application
/// D is the per interaction state thats passed down for each interaction
pub struct WidgetContainer<T, D> {
    children: Vec<WidgetContainer<T, D>>,
    widget: Box<dyn Widget<T, D>>,
    ratio: LogicalSize,
    alignment: ContainerAlignment,
}
impl<T, D> WidgetContainer<T, D> {
    pub fn root(widget: impl Widget<T, D> + 'static) -> WidgetContainer<T, D> {
        WidgetContainer::new(widget, LogicalSize::new(1.0, 1.0), ContainerAlignment::Left)
    }
    fn new(
        widget: impl Widget<T, D> + 'static,
        ratio: LogicalSize,
        alignment: ContainerAlignment,
    ) -> WidgetContainer<T, D> {
        WidgetContainer { children: vec![], widget: Box::new(widget), ratio, alignment }
    }

    pub fn add_child(
        &mut self,
        widget: impl Widget<T, D> + 'static,
        size: LogicalSize,
        alignment: ContainerAlignment,
    ) -> &mut Self {
        self.children.push(WidgetContainer::new(widget, size, alignment));
        self
    }

    pub fn with_child(&mut self, widget: impl Widget<T, D> + 'static, size: LogicalSize) -> &mut WidgetContainer<T, D> {
        self.children.push(WidgetContainer::new(widget, size, ContainerAlignment::Left));

        let count = self.children.len();
        &mut self.children[count - 1]
    }

    pub fn action(
        &mut self,
        state: &mut T,
        animations: &mut AnimationManager,
        action: &Actions,
        current_focus: &WidgetId,
        handler: &mut D,
    ) -> bool {
        //Only send action to currently focused widget
        let handled = current_focus == &self.widget.get_id() && self.widget.action(state, animations, action, handler);
        if handled {
            return true;
        }

        for i in self.children.iter_mut() {
            if i.action(state, animations, action, current_focus, handler) {
                return true;
            }
        }

        false
    }

    pub fn render(&mut self, state: &T, graphics: &mut Graphics, current_focus: &WidgetId) {
        //These measure the offset from the edges to begin or end rendering
        let mut top_stack = 0.;
        let mut bottom_stack = 0.;
        let mut left_stack = 0.;
        let mut right_stack = 0.;

        let rect = self.widget.layout();
        graphics.bounds = rect;
        self.widget.render(graphics, state, current_focus);

        for i in self.children.iter_mut() {
            let size = LogicalSize::new(rect.width() * i.ratio.x, rect.height() * i.ratio.y);

            let origin;
            match i.alignment {
                ContainerAlignment::Left => {
                    origin = LogicalPosition::new(rect.left() + left_stack, rect.top());
                    left_stack += size.x;
                }
                ContainerAlignment::Right => {
                    origin = LogicalPosition::new(rect.right() - (size.x + right_stack), rect.top());
                    right_stack += size.x;
                }
                ContainerAlignment::Top => {
                    origin = LogicalPosition::new(rect.left(), rect.top() + top_stack);
                    top_stack += size.y;
                }
                ContainerAlignment::Bottom => {
                    origin = LogicalPosition::new(rect.left(), rect.bottom() - (size.y + bottom_stack));
                    bottom_stack += size.y;
                }
            };

            let offset = i.widget.offset();
            let origin = LogicalPosition::new(origin.x + offset.x * size.x, origin.y + offset.y * size.y);
            let r = Rect::new(origin, origin + size);
            i.widget.set_layout(r);

            i.render(state, graphics, current_focus);
        }
    }

    fn find_widget_mut(&mut self, widget: WidgetId) -> Option<&mut WidgetContainer<T, D>> {
        let id = self.widget.get_id();
        if widget == id {
            return Some(self);
        }

        for i in self.children.iter_mut() {
            let widget = i.find_widget_mut(widget);
            if widget.is_some() {
                return widget;
            }
        }
        None
    }
}
