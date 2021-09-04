//https://docs.rs/piet/0.0.7/piet/trait.RenderContext.html
//https://github.com/linebender/druid/blob/master/druid/src/widget/image.rs
use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use speedy2d::color::Color;
use speedy2d::font::{FormattedTextBlock, TextLayout, TextOptions, TextAlignment};
use crate::{YaffeState, Actions, V2, Rect};
use std::ops::Deref;
use crate::widgets::animations::*;

pub mod animations;
mod platform_list;
mod app_list;
mod search_bar;
mod toolbar;
mod background;
mod app_tile;
mod info_pane;
pub use platform_list::PlatformList;
pub use app_list::AppList;
pub use search_bar::{SearchBar, SearchInfo};
pub use toolbar::Toolbar;
pub use background::Background;
pub use app_tile::AppTile;
pub use info_pane::InfoPane;

pub trait UiElement {
    fn position(&self) -> V2;
    fn size(&self) -> V2;
    fn layout(&self) -> Rectangle;
    fn set_layout(&mut self, layout: Rectangle);
}
pub type WidgetId = std::any::TypeId;
pub trait FocusableWidget: UiElement {
    fn get_id(&self) -> WidgetId;
}
pub trait Widget: FocusableWidget {
    /// Update and draw
    fn render(&mut self, state: &YaffeState, rect: Rectangle, delta_time: f32, piet: &mut Graphics2D);

    /// Offset from initial placement to move. Percentage based on widget size
    fn offset(&self) -> V2 { V2::new(0., 0.) }

    /// Called when a user action occurs
    fn action(&mut self, _: &mut YaffeState, _: &Actions, _: &mut DeferredAction) -> bool { false }

    /// Called when the control gets focus
    fn got_focus(&mut self, _: Rectangle, _: &mut DeferredAction) {}

    /// Called when the control loses focus
    fn lost_focus(&mut self, _: Rectangle, _: &mut DeferredAction) {}

    /// Called when a restricted action has been validated
    fn on_restricted_action_finalized(&self, _: &YaffeState, _: &'static str, _: &mut DeferredAction) {}
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
            #[allow(dead_code)]queue: std::sync::Arc<std::cell::RefCell<crate::JobQueue>>, 
            pub position: crate::V2,
            pub size: crate::V2,
            $($element: $ty),* 
        }
        impl crate::widgets::UiElement for $name {
            fn position(&self) -> crate::V2 { self.position }
            fn size(&self) -> crate::V2 { self.size }
            fn layout(&self) -> Rectangle { Rectangle::new(self.position, self.position + self.size) }
            fn set_layout(&mut self, layout: Rectangle) { 
                self.position = *layout.top_left(); 
                self.size = layout.size();
            }
        }
        impl crate::widgets::FocusableWidget for $name {
            fn get_id(&self) -> crate::widgets::WidgetId { std::any::TypeId::of::<$name>() }
        }
        impl $name {
            pub fn new(q: std::sync::Arc<std::cell::RefCell<crate::JobQueue>>) -> $name {
                $name { 
                    queue: q, 
                    position: crate::V2::new(0., 0.),
                    size: crate::V2::new(0., 0.),
                    $($element: $value),*
                }
            }
        }
    };
}

#[repr(u8)]
pub enum ContainerAlignment {
    Left,
    Right,
    Top,
    Bottom,
}

/// Container for our widgets that lays them out in the tree
/// Has higher level management methods to perfrom things
/// on the entire UI tree
pub struct WidgetTree {
    pub root: WidgetContainer,
    pub focus: Vec<WidgetId>,
    pub data: YaffeState,
    pub anims: Vec<Animation>,
    pub layout_valid: bool,
}
impl WidgetTree {
    pub fn new(root: WidgetContainer, data: YaffeState) -> WidgetTree {
        WidgetTree {
            root: root,
            focus: vec!(),
            data: data,
            anims: vec!(),
            layout_valid: false,
        }
    }

    pub fn render_all(&mut self, layout: Rectangle, piet: &mut Graphics2D, delta_time: f32, invalidate: bool) {
        if invalidate { self.root.widget.set_layout(layout); }
        self.root.render(&self.data, piet, delta_time, invalidate);
    }

    fn current_focus(&mut self) -> Option<&mut WidgetContainer> {
        if let Some(last) = self.focus.last() {
            return self.root.find_widget_mut(*last);
        }
        None
    }

    pub fn focus(&mut self, widget: WidgetId) {
        let mut handle = DeferredAction::new();
        //Find current focus so we can notify it is about to lose
        if let Some(lost) = self.current_focus() {
            lost.widget.lost_focus(lost.original_layout.clone(), &mut handle);
        }
    
        //Find new focus
        if let Some(got) = self.root.find_widget_mut(widget) {
            got.widget.got_focus(got.original_layout.clone(), &mut handle);
            self.focus.push(widget);
        }

        handle.resolve(self);
    }

    fn revert_focus(&mut self) {
        let mut handle = DeferredAction::new();
        //Find current focus so we can notify it is about to lose
        if let Some(last) = self.focus.pop() {
            if let Some(lost) = self.root.find_widget_mut(last) {
                lost.widget.lost_focus(lost.original_layout.clone(), &mut handle);
            }
        }

        //Revert to previous focus
        if let Some(got) = self.current_focus() {
            got.widget.got_focus(got.original_layout.clone(), &mut handle);
        }

        handle.resolve(self);
    }

    fn finalize_restricted_action(&mut self, tag: &'static str) {
        if crate::restrictions::try_disable_restrictions(&mut self.data, tag) { return; }

        let mut handle = DeferredAction::new();
        if let Some(last) = self.focus.last() {
            if let Some(focus) = self.find_widget(*last) {
                focus.widget.on_restricted_action_finalized(&self.data, tag, &mut handle);
            }
        }
        handle.resolve(self);
    }
}

impl Deref for WidgetTree {
    type Target = WidgetContainer;
    fn deref(&self) -> &Self::Target {
        &self.root
    }
}

/// Contains an individual widget
/// Manages ratio sizing and animations
pub struct WidgetContainer {
    children: Vec<WidgetContainer>,
    widget: Box<dyn Widget>,
    ratio: V2,
    original_layout: Rectangle,
    alignment: ContainerAlignment,
}
impl WidgetContainer {
    pub fn root(widget: impl Widget + 'static) -> WidgetContainer {
        WidgetContainer {
            children: vec!(),
            widget: Box::new(widget),
            ratio: V2::new(1.0, 1.0),
            original_layout: Rectangle::from_tuples((0., 0.), (0., 0.)),
            alignment: ContainerAlignment::Left,
        }
    }
    fn new(widget: impl Widget + 'static, size: V2, alignment: ContainerAlignment) -> WidgetContainer {
         WidgetContainer {
            children: vec!(),
            widget: Box::new(widget),
            ratio: size,
            original_layout: Rectangle::from_tuples((0., 0.), (0., 0.)),
            alignment: alignment,
         }
    }

    pub fn add_child(&mut self, widget: impl Widget + 'static, size: V2, alignment: ContainerAlignment) -> &mut Self {
        self.children.push(WidgetContainer::new(widget, size, alignment));
        self
    }

    pub fn with_child(&mut self, widget: impl Widget + 'static, size: V2) -> &mut WidgetContainer {
        self.children.push(WidgetContainer::new(widget, size, ContainerAlignment::Left));

        let count = self.children.len();
        &mut self.children[count - 1]
    }

    pub fn action(&mut self, state: &mut YaffeState, action: &Actions, current_focus: &WidgetId, handler: &mut DeferredAction) -> bool {
        //Only send action to currently focused widget
        let handled = current_focus == &self.widget.get_id() && self.widget.action(state, action, handler);

        if !handled {
            for i in self.children.iter_mut() {
                let handled = i.action(state, action, current_focus, handler);
                if handled { break; }
            }
        }

        handled
    }

    pub fn render(&mut self, state: &YaffeState, piet: &mut Graphics2D, delta_time: f32, invalidate: bool) {
        let mut top_stack = 0.;
        let mut bottom_stack = 0.;
        let mut left_stack = 0.;
        let mut right_stack = 0.;
        let rect = self.widget.layout();
        self.widget.render(state, rect.clone(), delta_time, piet);

        for i in self.children.iter_mut() {
            if invalidate {
                let size = V2::new(rect.width() * i.ratio.x, rect.height() * i.ratio.y);

                let origin;
                match i.alignment {
                    ContainerAlignment::Left => {
                        origin = V2::new(rect.left() + left_stack, rect.top());
                        left_stack += size.x;
                    },
                    ContainerAlignment::Right => {
                        origin = V2::new(rect.right() - (size.x + right_stack), rect.top());
                        right_stack += size.x;
                    },
                    ContainerAlignment::Top => {
                        origin = V2::new(rect.left(), rect.top() + top_stack);
                        top_stack += size.y;
                    },
                    ContainerAlignment::Bottom => {
                        origin = V2::new(rect.left(), rect.bottom() - (size.y + bottom_stack));
                        bottom_stack += size.y;
                    },
                };

                let offset = i.widget.offset();
                let origin = V2::new(origin.x + offset.x * size.x, origin.y + offset.y * size.y);
                let r = Rectangle::new(origin, origin + size);
                i.original_layout = r.clone();
                i.widget.set_layout(r);
            }
            i.render(state, piet, delta_time, invalidate);
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

    fn find_widget(&self, widget: WidgetId) -> Option<&WidgetContainer> {
        let id = self.widget.get_id();
        if widget == id { return Some(self) }

        for i in self.children.iter() {
            let widget = i.find_widget(widget);
            if widget.is_some() { return widget; }
        }
        None
    }
}


#[repr(u8)]
enum FocusType {
    Revert,
    Focus(WidgetId),
}

pub struct DeferredAction {
    focus: Option<FocusType>,
    anims: Vec<Animation>,
    load_plugin: Option<crate::plugins::PluginLoadType>,
    restricted_action: Option<&'static str>,
    message: Option<String>,
}
impl DeferredAction {
    pub fn new() -> DeferredAction {
        DeferredAction { 
            focus: None,
            load_plugin: None,
            anims: vec!(),
            restricted_action: None,
            message: None,
        }
    }
    pub fn focus_widget(&mut self, widget: WidgetId) {
        self.focus = Some(FocusType::Focus(widget));
    }
    pub fn revert_focus(&mut self) {
        self.focus = Some(FocusType::Revert);
    }
    pub fn load_plugin(&mut self, kind: crate::plugins::PluginLoadType) {
        self.load_plugin = Some(kind);
    }
    pub fn finalize_restricted_action(&mut self, tag: &'static str) {
        self.restricted_action = Some(tag);
    }
    pub fn display_message(&mut self, message: String) {
        self.message = Some(message);
    }

    pub fn resolve(self, ui: &mut WidgetTree) {
        if let Some(tag) = self.restricted_action {
            ui.finalize_restricted_action(tag);
        }

        //TODO track revert focus time for a hard revert?
        match self.focus {
            None => { /*do nothing*/ }
            Some(FocusType::Revert) => ui.revert_focus(),
            Some(FocusType::Focus(w)) => ui.focus(w),
        }

        //Update any animations
        for i in self.anims {
            ui.anims.push(i);
        }

        if let Some(kind) = self.load_plugin {
            let state = &mut ui.data;
            crate::plugins::load_plugin_items(kind, state);
        }

        if let Some(message) = self.message {
            let message = Box::new(crate::modals::MessageModalContent::new(&message));
            crate::modals::display_modal(&mut ui.data, "Error", None, message, crate::modals::ModalSize::Half, None);
        }
    }
}

//
// Text helper methods
//
/// Draws text that is right aligned to parameter `right`
/// If an image is passed it will be drawn to the left of the text
/// Returns the new right-most position
pub fn right_aligned_text(piet: &mut Graphics2D, right: V2, image: Option<crate::assets::Images>, color: Color, text: std::rc::Rc<FormattedTextBlock>) -> V2 {
    let size = V2::new(text.width(), text.height());
    let mut right = V2::new(right.x - size.x, right.y);

    piet.draw_text(right, color, &text);
    if let Some(i) = image {
        right.x -= size.y;
        let i = crate::assets::request_preloaded_image(piet, i);
        i.render(piet, Rectangle::new(right, right + V2::new(size.y, size.y)));
    }

    right
}

/// Simple helper method to get a text object
pub fn get_drawable_text(size: f32, text: &str) -> std::rc::Rc<FormattedTextBlock> {
    let font = crate::assets::request_font(crate::assets::Fonts::Regular);
    font.layout_text(text, size, TextOptions::new())
}

/// Simple helper method to get a text object that is wrapped to a certain size
pub fn get_drawable_text_with_wrap(size: f32, text: &str, width: f32) -> std::rc::Rc<FormattedTextBlock> {
    let font =  crate::assets::request_font(crate::assets::Fonts::Regular);
    let option = TextOptions::new();
    let option = option.with_wrap_to_width(width, TextAlignment::Left);
    font.layout_text(text, size, option)
}

trait Shifter {
    fn shift_x(&self, amount: f32) -> Self;
}
impl Shifter for V2 {
    fn shift_x(&self, amount: f32) -> Self {
        V2::new(self.x + amount, self.y)
    }
}