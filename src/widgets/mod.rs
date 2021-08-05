//https://docs.rs/piet/0.0.7/piet/trait.RenderContext.html
//https://github.com/linebender/druid/blob/master/druid/src/widget/image.rs
use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use speedy2d::color::Color;
use speedy2d::font::{FormattedTextBlock, TextLayout, TextOptions, TextAlignment};
use crate::{YaffeState, Actions, V2, Rect};
use std::ops::Deref;

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

    /// Allows the widget to position and size itself according to the parent widget
    fn place(&self, space: &Rectangle, size: V2) -> Rectangle { 
        Rectangle::from_tuples((space.left(), space.top()), (space.left() + size.x, space.top() + size.y))
    }
    
    /// Called when a user action occurs
    fn action(&mut self, _: &mut YaffeState, _: &Actions, _: &mut DeferredAction) -> bool { false }

    /// Called when the control gets focus
    fn got_focus(&mut self, _: &mut DeferredAction) {}

    /// Called when the control loses focus
    fn lost_focus(&mut self, _: &mut DeferredAction) {}
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
            layout: Rectangle,
            $($element: $ty),* 
        }
        impl crate::widgets::UiElement for $name {
            fn position(&self) -> crate::V2 { *self.layout.top_left() }
            fn size(&self) -> crate::V2 { self.layout.size() }
            fn layout(&self) -> Rectangle { self.layout.clone() }
            fn set_layout(&mut self, layout: Rectangle) { self.layout = layout; }
        }
        impl crate::widgets::FocusableWidget for $name {
            fn get_id(&self) -> crate::widgets::WidgetId { std::any::TypeId::of::<$name>() }
        }
        impl $name {
            pub fn new(q: std::sync::Arc<std::cell::RefCell<crate::JobQueue>>) -> $name {
                $name { 
                    queue: q, 
                    layout: Rectangle::from_tuples((0., 0.), (0., 0.)),
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
    pub anims: std::collections::HashMap<WidgetId, Animation>,
    pub layout_valid: bool,
}
impl WidgetTree {
    pub fn new(root: WidgetContainer, data: YaffeState) -> WidgetTree {
        WidgetTree {
            root: root,
            focus: vec!(),
            data: data,
            anims: std::collections::HashMap::new(),
            layout_valid: false,
        }
    }

    pub fn render_all(&mut self, layout: Rectangle, piet: &mut Graphics2D, delta_time: f32, invalidate: bool) {
        if invalidate {
            let size = V2::new(layout.width() * self.root.ratio.x, layout.height() * self.root.ratio.y);
            let r = self.root.widget.place(&layout, size);
            self.root.widget.set_layout(r);
        }
        self.root.render(&self.data, self.root.widget.layout().clone(), piet, delta_time, invalidate);
    }

    pub fn focus(&mut self, widget: WidgetId) {
        let mut handle = DeferredAction::new();
        //Find current focus so we can notify it is about to lose
        if let Some(last) = self.focus.last() {
            if let Some(lost) = self.root.find_widget(*last) {
                lost.widget.lost_focus(&mut handle);
            }
        }
        
        //Find new focus
        if let Some(got) = self.root.find_widget(widget) {
            got.widget.got_focus(&mut handle);
            self.focus.push(widget);
        }

        //Update any animations
        for i in handle.anims {
            if self.anims.contains_key(&i.widget) {
                self.anims.remove(&i.widget);
            }
            self.anims.insert(i.widget, i);
        }
    }

    pub fn revert_focus(&mut self) {
        let mut handle = DeferredAction::new();
        //Find current focus so we can notify it is about to lose
        if let Some(last) = self.focus.pop() {
            if let Some(lost) = self.root.find_widget(last) {
                lost.widget.lost_focus(&mut handle);
            }
        }

        //Revert to previous focus
        if let Some(f) = self.focus.last() {
            if let Some(got) = self.root.find_widget(*f) {
                got.widget.got_focus(&mut handle);
            }
        }

        //Update any animations
        for i in handle.anims {
            if self.anims.contains_key(&i.widget) {
                self.anims.remove(&i.widget);
            }
            self.anims.insert(i.widget, i);
        }
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
    orientation: ContainerAlignment,
}
impl WidgetContainer {
    pub fn root(widget: impl Widget + 'static) -> WidgetContainer {
        WidgetContainer {
            children: vec!(),
            widget: Box::new(widget),
            ratio: V2::new(1.0, 1.0),
            orientation: ContainerAlignment::Left,
        }
    }
    fn new(widget: impl Widget + 'static, size: V2) -> WidgetContainer {
         WidgetContainer {
            children: vec!(),
            widget: Box::new(widget),
            ratio: size,
            orientation: ContainerAlignment::Left,
         }
    }

    fn set_position(&mut self, rect: V2) {
        let layout = self.widget.layout();
        self.widget.set_layout(Rectangle::new(rect, rect + layout.size()))
    }

    pub fn alignment(&mut self, orientation: ContainerAlignment) -> &mut Self {
        self.orientation = orientation;
        self
    }

    pub fn add_child(&mut self, widget: impl Widget + 'static, size: V2) -> &mut Self {
        self.children.push(WidgetContainer::new(widget, size));
        self
    }

    pub fn with_child(&mut self, widget: impl Widget + 'static, size: V2) -> &mut WidgetContainer {
        self.children.push(WidgetContainer::new(widget, size));

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

    pub fn render(&mut self, state: &YaffeState, rect: Rectangle, piet: &mut Graphics2D, delta_time: f32, invalidate: bool) {
        let mut x = rect.left();
        let y = rect.top();
        self.widget.render(state, self.widget.layout(), delta_time, piet);

        for i in self.children.iter_mut() {
            if invalidate {
                let size = V2::new(rect.width() * i.ratio.x, rect.height() * i.ratio.y);
                let r = i.widget.place(&Rectangle::from_tuples((x, y), (rect.right(), rect.bottom())), size);
                i.widget.set_layout(r);
            }
            i.render(state, i.widget.layout().clone(), piet, delta_time, invalidate);

            match self.orientation {
                ContainerAlignment::Left => x += i.widget.layout().width(),
                ContainerAlignment::Right => { /* do nothing. child must position itself */ }
                ContainerAlignment::Top => {},
                ContainerAlignment::Bottom => {},
            }
        }
    }

    fn find_widget(&mut self, widget: WidgetId) -> Option<&mut WidgetContainer> {
        let id = self.widget.get_id();
        if widget == id { return Some(self) }

        for i in self.children.iter_mut() {
            let widget = i.find_widget(widget);
            if widget.is_some() { return widget; }
        }
        None
    }
}

//
// Actions/animations
//
enum AnimationType {
    Position(V2),
    Placeholder,
}

pub struct Animation {
    widget: WidgetId,
    anim: AnimationType,
    duration: f32,
    remaining: f32,
}

#[repr(u8)]
enum FocusType {
    Revert,
    Focus(WidgetId),
}

pub struct DeferredAction {
    focus: Option<FocusType>,
    anims: Vec<Animation>,
}
impl DeferredAction {
    pub fn new() -> DeferredAction {
        DeferredAction { 
            focus: None,
            anims: vec!(),
        }
    }
    pub fn focus_widget(&mut self, widget: WidgetId) {
        self.focus = Some(FocusType::Focus(widget));
    }
    pub fn revert_focus(&mut self) {
        self.focus = Some(FocusType::Revert);
    }

    pub fn resolve(self, ui: &mut WidgetTree) {
        match self.focus {
            None => { /*do nothing*/ }
            Some(FocusType::Revert) => ui.revert_focus(),
            Some(FocusType::Focus(w)) => ui.focus(w),
        }

        //Update any animations
        for i in self.anims {
            if ui.anims.contains_key(&i.widget) {
                ui.anims.remove(&i.widget);
            }
            ui.anims.insert(i.widget, i);
        }
    }

    pub fn animate(&mut self, widget: &impl FocusableWidget, to: V2, duration: f32) {
        self.anims.push(Animation {
            widget: widget.get_id(),
            anim: AnimationType::Position(to),
            duration: duration,
            remaining: duration,
        });
    }

    pub fn animate_placeholder(&mut self, duration: f32) {
        self.anims.push(Animation {
            widget: std::any::TypeId::of::<crate::widgets::app_tile::AppTile>(),
            anim: AnimationType::Placeholder,
            duration: duration,
            remaining: duration,
        });
    }
}

/// Processes any widgets that have running animations
/// Currently only position animations are allowed
pub fn run_animations(tree: &mut WidgetTree, delta_time: f32) {
    let mut keys = vec!();

    fn lerp(from: V2, to: V2, amount: f32) -> V2 {
        V2::new(from.x + amount * (to.x - from.x), from.y + amount * (to.y - from.y))
    }

    //Run animations, if it completes, mark it for removal
    for (k, animation) in tree.anims.iter_mut() {
        if animation.remaining > 0. { animation.remaining = f32::max(0., animation.remaining - delta_time);}
        else if animation.remaining == 0. { animation.remaining -= delta_time; }
        
        match animation.anim {
            AnimationType::Position(to) => {
                if let Some(widget) = tree.root.find_widget(animation.widget) {

                    //TODO the lerping causes some jank at the end of animations
                    let from = widget.widget.position();
                    widget.set_position(lerp(from, to, delta_time / animation.duration)); 
            
                    if animation.remaining == 0. { widget.set_position(to); }
                }
            },

            AnimationType::Placeholder => { }
        }

        if animation.remaining < 0. {
            keys.push(k.clone());
         }
    }

    for k in keys {
        tree.anims.remove(&k);
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
fn get_drawable_text_with_wrap(size: f32, text: &str, width: f32) -> std::rc::Rc<FormattedTextBlock> {
    let font = crate::assets::request_font(crate::assets::Fonts::Regular);
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