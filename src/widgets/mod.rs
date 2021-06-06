//https://docs.rs/piet/0.0.7/piet/trait.RenderContext.html
//https://github.com/linebender/druid/blob/master/druid/src/widget/image.rs
use druid_shell::kurbo::{Size, Rect, Point};
use druid_shell::piet::{Piet, RenderContext, Text, TextLayout, PietTextLayout, 
                        TextLayoutBuilder, TextAlignment, Color };
use crate::{YaffeState, Actions};
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

pub type WidgetId = std::any::TypeId;
pub trait WidgetName {
    fn get_id(&self) -> WidgetId;
}
pub trait Widget: WidgetName {
    /// Update and draw
    fn render(&mut self, state: &YaffeState, rect: Rect, piet: &mut Piet);

    /// Allows the widget to position and size itself according to the parent widget
    fn layout(&self, space: &Rect, size: Size) -> Rect { 
        Rect::new(space.x0, space.y0, space.x0 + size.width, space.y0 + size.height)
    }
    
    /// Called when a user action occurs
    fn action(&mut self, _: &mut YaffeState, _: &Actions, _: &mut DeferredAction) -> bool { false }

    /// Called when the control gets focus
    fn got_focus(&mut self, _: &Rect, _: &mut DeferredAction) {}

    /// Called when the control loses focus
    fn lost_focus(&mut self, _: &Rect, _: &mut DeferredAction) {}
}

#[macro_export]
macro_rules! get_widget_id {
    ($widget:ty) => {
        std::any::TypeId::of::<$widget>()
    };
}

#[macro_export]
macro_rules! create_widget {
    ($name:ident, $($element:ident: $ty:ty = $value:expr),*) => {
        #[allow(unused_variables)]
        pub struct $name { #[allow(dead_code)]queue: std::sync::Arc<std::cell::RefCell<crate::JobQueue>>, $($element: $ty),* }
        impl crate::widgets::WidgetName for $name {
            fn get_id(&self) -> crate::widgets::WidgetId { std::any::TypeId::of::<$name>() }
        }
        impl $name {
            pub fn new(q: std::sync::Arc<std::cell::RefCell<crate::JobQueue>>) -> $name {
                $name { 
                    queue: q, 
                    $($element: $value),*
                }
            }
        }
    };
}

#[repr(u8)]
pub enum ContainerOrientation {
    Horizontal,
    Floating
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

    pub fn render_all(&mut self, layout: Rect, piet: &mut Piet, invalidate: bool) {
        if invalidate {
            let size = Size::new(layout.width() * self.root.ratio.width, layout.height() * self.root.ratio.height);
            let r = self.root.data.layout(&layout, size);
            self.root.set_layout(r);
        }
        self.root.render(&self.data, self.root.layout, piet, invalidate);
    }

    pub fn focus(&mut self, widget: WidgetId) {
        let mut handle = DeferredAction::new();
        //Find current focus so we can notify it is about to lose
        if let Some(last) = self.focus.last() {
            if let Some(lost) = self.root.find_widget(*last) {
                lost.data.lost_focus(&lost.layout, &mut handle);
            }
        }
        
        //Find new focus
        if let Some(got) = self.root.find_widget(widget) {
            got.data.got_focus(&got.layout, &mut handle);
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
                lost.data.lost_focus(&lost.layout, &mut handle);
            }
        }

        //Revert to previous focus
        if let Some(f) = self.focus.last() {
            if let Some(got) = self.root.find_widget(*f) {
                got.data.got_focus(&got.layout, &mut handle);
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
    data: Box<dyn Widget>,
    layout: Rect,
    pos: Point,
    size: Size,
    ratio: Size,
    orientation: ContainerOrientation,
}
impl WidgetContainer {
    pub fn root(widget: impl Widget + 'static) -> WidgetContainer {
        WidgetContainer {
            children: vec!(),
            data: Box::new(widget),
            layout: Rect::new(0., 0., 0., 0.),
            pos: Point::new(0., 0.),
            size: Size::new(0., 0.),
            ratio: Size::new(1.0, 1.0),
            orientation: ContainerOrientation::Horizontal,
        }
    }
    fn new(widget: impl Widget + 'static, size: Size) -> WidgetContainer {
         WidgetContainer {
            children: vec!(),
            data: Box::new(widget),
            layout: Rect::new(0., 0., 0., 0.),
            pos: Point::new(0., 0.),
            size: Size::new(0., 0.),
            ratio: size,
            orientation: ContainerOrientation::Horizontal,
         }
    }

    fn set_layout(&mut self, rect: Rect) {
        self.layout = rect;
        self.size = rect.size();
        self.pos = rect.origin();
    }

    pub fn orientation(&mut self, orientation: ContainerOrientation) -> &mut Self {
        self.orientation = orientation;
        self
    }

    pub fn add_child(&mut self, widget: impl Widget + 'static, size: Size) -> &mut Self {
        self.children.push(WidgetContainer::new(widget, size));
        self
    }

    pub fn with_child(&mut self, widget: impl Widget + 'static, size: Size) -> &mut WidgetContainer {
        self.children.push(WidgetContainer::new(widget, size));

        let count = self.children.len();
        &mut self.children[count - 1]
    }

    pub fn action(&mut self, state: &mut YaffeState, action: &Actions, current_focus: &WidgetId, handler: &mut DeferredAction) -> bool {
        //Only send action to currently focused widget
        let handled = current_focus == &self.data.get_id() && self.data.action(state, action, handler);

        if !handled {
            for i in self.children.iter_mut() {
                let handled = i.action(state, action, current_focus, handler);
                if handled { break; }
            }
        }

        handled
    }

    pub fn render(&mut self, state: &YaffeState, rect: Rect, piet: &mut Piet, invalidate: bool) {
        let mut x = rect.x0;
        let y = rect.y0;
        self.data.render(state, Rect::from((self.pos, self.size)), piet);

        for i in self.children.iter_mut() {
            if invalidate {
                let size = Size::new(rect.width() * i.ratio.width, rect.height() * i.ratio.height);
                let r = i.data.layout(&Rect::new(x, y, rect.x1, rect.y1), size);
                i.set_layout(r);
            }
            i.render(state, i.layout, piet, invalidate);

            match self.orientation {
                ContainerOrientation::Horizontal => x += i.layout.width(),
                ContainerOrientation::Floating => { /* do nothing. child must position itself */ }
            }
        }
    }

    fn find_widget(&mut self, widget: WidgetId) -> Option<&mut WidgetContainer> {
        let id = self.data.get_id();
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
pub struct Animation {
    pub widget: WidgetId,
    to: Point,
    duration: f64,
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

    pub fn animate(&mut self, widget: &impl WidgetName, to: Point, duration: f64) {
        self.anims.push(Animation {
            widget: widget.get_id(),
            to: to,
            duration: duration,
        });
    }
}

/// Processes any widgets that have running animations
/// Currently only position animations are allowed
pub fn run_animations(tree: &mut WidgetTree, delta_time: f64) {
    let mut keys = vec!();
    
    //Run animations, if it completes, mark it for removal
    for (k, a) in tree.anims.iter_mut() {
        if let Some(widget) = tree.root.find_widget(a.widget) {
            let from = widget.pos;
            widget.pos = from.lerp(a.to, delta_time / a.duration);

            if from.distance(a.to) < 1.{
               widget.pos = a.to;
               keys.push(k.clone());
            }
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
pub fn right_aligned_text(piet: &mut Piet, right: Point, image: Option<crate::assets::Images>, text: PietTextLayout) -> Point {
    let size = text.size();
    let mut right = Point::new(right.x - size.width, right.y);

    piet.draw_text(&text, right);
    if let Some(i) = image {
        right.x -= size.height;
        let i = crate::assets::request_preloaded_image(piet, i);
        i.render(piet, Rect::from((right, Size::new(size.height, size.height))));
    }

    right
}

/// Simple helper method to get a text object
pub fn get_drawable_text(piet: &mut Piet, size: f64, text: &str, color: Color) -> PietTextLayout {
    let font = crate::assets::request_font(piet);
    piet
        .text()
        .new_text_layout(String::from(text))
        .font(font, size)
        .text_color(color)
        .alignment(TextAlignment::Start)
        .build()
        .unwrap()
}

/// Simple helper method to get a text object that is wrapped to a certain size
fn get_drawable_text_with_wrap(piet: &mut Piet, size: f64, text: &str, color: Color, width: f64) -> PietTextLayout {
    let font = crate::assets::request_font(piet);
    piet
        .text()
        .new_text_layout(String::from(text))
        .font(font, size)
        .text_color(color)
        .alignment(TextAlignment::Start)
        .max_width(width)
        .build()
        .unwrap()
}

trait Shifter {
    fn shift_x(&self, amount: f64) -> Self;
}
impl Shifter for Point {
    fn shift_x(&self, amount: f64) -> Self {
        Point::new(self.x + amount, self.y)
    }
}