use speedy2d::color::Color;
use speedy2d::font::{FormattedTextBlock, TextLayout, TextOptions, TextAlignment};
use crate::{YaffeState, Graphics, Actions, LogicalPosition, LogicalSize, ScaleFactor, Rect};
use crate::widgets::animations::*;
use crate::assets::{Images, Fonts, AssetSlot};
use std::ops::Deref;
use std::time::Instant;

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
    fn got_focus(&mut self, _: Rect, _: &mut DeferredAction) {}

    /// Called when the control loses focus
    fn lost_focus(&mut self, _: Rect, _: &mut DeferredAction) {}
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
            pub position: crate::LogicalPosition,
            pub size: crate::LogicalSize,
            $($element: $ty),* 
        }
        impl crate::widgets::UiElement for $name {
            fn position(&self) -> crate::LogicalPosition { self.position }
            fn size(&self) -> crate::LogicalSize { self.size }
            fn layout(&self) -> crate::Rect { crate::Rect::new(self.position, self.position + self.size) }
            fn set_layout(&mut self, layout: crate::Rect) { 
                self.position = *layout.top_left(); 
                self.size = layout.size();
            }
        }
        impl crate::widgets::FocusableWidget for $name {
            fn get_id(&self) -> crate::widgets::WidgetId { std::any::TypeId::of::<$name>() }
        }
        impl $name {
            pub fn new() -> $name {
                $name { 
                    position: crate::LogicalPosition::new(0., 0.),
                    size: crate::LogicalSize::new(0., 0.),
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
    anims: Vec<Animation>,
    layout_valid: bool,
    last_focus: (Option<WidgetId>, Instant),
}
impl WidgetTree {
    pub fn new(root: WidgetContainer, data: YaffeState) -> WidgetTree {
        WidgetTree {
            root: root,
            focus: vec!(),
            data: data,
            anims: vec!(),
            layout_valid: false,
            last_focus: (None, Instant::now()),
        }
    }

    pub fn render_all(&mut self, graphics: &mut crate::Graphics) {
        if !self.layout_valid { self.root.widget.set_layout(graphics.bounds); }
        self.root.render(&self.data, graphics, !self.layout_valid);
        self.layout_valid = true;
    }

    pub fn invalidate(&mut self) {
        self.layout_valid = false;
    }

    pub fn needs_new_frame(&self) -> bool {
        self.anims.len() > 0
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
            self.last_focus = (Some(lost.widget.get_id()), Instant::now());
        }
    
        //Find new focus
        if let Some(got) = self.root.find_widget_mut(widget) {
            got.widget.got_focus(got.original_layout.clone(), &mut handle);
            self.focus.push(widget);
        }

        handle.resolve(self);
    }

    fn revert_focus(&mut self) {
        let now = Instant::now();

        //Check if we have pressed back multiple times in quick succession
        //If we have revert all the way to the last different widget
        //This will allow us to get back to the platform list after going deep in a plugin
        //items
        let mut last = self.focus.pop();
        if (now - self.last_focus.1).as_millis() < 200 {
            while last.as_ref() == self.focus.last() {
                last = self.focus.pop();
            }
        }
        let different = last != self.last_focus.0;
        self.last_focus = (last, now);
        
        let mut handle = DeferredAction::new();
        //Find current focus so we can notify it is about to lose
        if let Some(last) = last {
            if let Some(lost) = self.root.find_widget_mut(last) {
                lost.widget.lost_focus(lost.original_layout.clone(), &mut handle);
            }
        }

        //Revert to previous focus
        if let Some(got) = self.current_focus() {
            got.widget.got_focus(got.original_layout.clone(), &mut handle);
        }

        if !different {
            //The only scenario this could happen is plugins
            handle.load_plugin(crate::plugins::NavigationAction::Back);
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
    ratio: LogicalSize,
    original_layout: Rect,
    alignment: ContainerAlignment,
}
impl WidgetContainer {
    pub fn root(widget: impl Widget + 'static) -> WidgetContainer {
        WidgetContainer::new(widget, LogicalSize::new(1.0, 1.0), ContainerAlignment::Left)
    }
    fn new(widget: impl Widget + 'static, size: LogicalSize, alignment: ContainerAlignment) -> WidgetContainer {
         WidgetContainer {
            children: vec!(),
            widget: Box::new(widget),
            ratio: size,
            original_layout: Rect::from_tuples((0., 0.), (0., 0.)),
            alignment: alignment,
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

    pub fn render(&mut self, state: &YaffeState, graphics: &mut Graphics, invalidate: bool) {
        //These measure the offset from the edges to begin or end rendering
        let mut top_stack = 0.;
        let mut bottom_stack = 0.;
        let mut left_stack = 0.;
        let mut right_stack = 0.;

        let rect = self.widget.layout();
        graphics.bounds = rect;
        self.widget.render(graphics, state);

        for i in self.children.iter_mut() {
            if invalidate {
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
                let r = Rect::new(origin.into(), (origin + size).into());
                i.original_layout = r.clone();
                i.widget.set_layout(r);
            }
            i.render(state, graphics, invalidate);
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


#[repr(u8)]
enum FocusType {
    Revert,
    Focus(WidgetId),
}

pub struct DeferredAction {
    focus: Option<FocusType>,
    anims: Vec<Animation>,
    load_plugin: Option<crate::plugins::NavigationAction>,
    message: Option<String>,
}
impl DeferredAction {
    pub fn new() -> DeferredAction {
        DeferredAction { 
            focus: None,
            load_plugin: None,
            anims: vec!(),
            message: None,
        }
    }
    pub fn focus_widget(&mut self, widget: WidgetId) {
        self.focus = Some(FocusType::Focus(widget));
    }
    pub fn revert_focus(&mut self) {
        self.focus = Some(FocusType::Revert);
    }
    pub fn load_plugin(&mut self, kind: crate::plugins::NavigationAction) {
        self.load_plugin = Some(kind);
    }
    pub fn display_message(&mut self, message: String) {
        self.message = Some(message);
    }

    pub fn resolve(self, ui: &mut WidgetTree) {
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
            crate::modals::display_modal(&mut ui.data, "Error", None, message, None);
        }
    }
}

//
// Text helper methods
//
/// Draws text that is right aligned to parameter `right`
/// If an image is passed it will be drawn to the left of the text
/// Returns the new right-most position
pub fn right_aligned_text(graphics: &mut crate::Graphics, right: LogicalPosition, image: Option<crate::assets::Images>, color: Color, text: std::rc::Rc<FormattedTextBlock>) -> LogicalPosition {
    let size = LogicalSize::new(text.width().to_logical(graphics), text.height().to_logical(graphics));
    let mut right = LogicalPosition::new(right.x - size.x, right.y);

    graphics.draw_text(right, color, &text);
    if let Some(i) = image {
        right.x -= size.y;
        let i = crate::assets::request_image(graphics, i).unwrap();
        i.render(graphics, Rect::point_and_size(right, LogicalSize::new(size.y, size.y)));
    }

    right
}

/// Simple helper method to get a text object
pub fn get_drawable_text(size: f32, text: &str) -> std::rc::Rc<FormattedTextBlock> {
    let font = crate::assets::request_font(Fonts::Regular);
    font.layout_text(text, size, TextOptions::new())
}

/// Simple helper method to get a text object that is wrapped to a certain size
pub fn get_drawable_text_with_wrap(size: f32, text: &str, width: f32) -> std::rc::Rc<FormattedTextBlock> {
    let font =  crate::assets::request_font(Fonts::Regular);
    let option = TextOptions::new();
    let option = option.with_wrap_to_width(width, TextAlignment::Left);
    font.layout_text(text, size, option)
}

/// Scales an image to the largest size that can fit in the smallest dimension
pub fn image_fill(graphics: &mut crate::Graphics, slot: &mut AssetSlot, size: &LogicalSize, expand: bool) -> LogicalSize {
    let mut tile_size = *size;
    
    let bitmap_size = if let Some(i) = crate::assets::request_asset_image(graphics, slot) {
            i.size()
    } else {
        crate::assets::request_image(graphics, Images::Placeholder).unwrap().size()
    };

    //By default on the recents menu it chooses the widest game boxart (see pFindMax in GetTileSize)
    //We wouldn't want vertical boxart to stretch to the horizontal dimensions
    //This will scale boxart that is different aspect to fit within the tile_size.Height
    let bitmap_size = bitmap_size.to_logical(graphics.scale_factor);
    let real_aspect = bitmap_size.x / bitmap_size.y;
    let tile_aspect = tile_size.x / tile_size.y;

    //If an aspect is wider than it is tall, it is > 1
    //If the two aspect ratios are on other sides of one, it means we need to scale
    if f32::is_sign_positive(real_aspect - 1.) != f32::is_sign_positive(tile_aspect - 1.) {
        //TODO this doesn't work
        tile_size.x = if expand { tile_size.y * real_aspect } else { tile_size.x * real_aspect };
    }

    tile_size
}

trait Shifter {
    fn shift_x(&self, amount: f32) -> Self;
}
impl Shifter for LogicalPosition {
    fn shift_x(&self, amount: f32) -> Self {
        LogicalPosition::new(self.x + amount, self.y)
    }
}