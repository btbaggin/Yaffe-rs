use crate::ui::{AnimationManager, WidgetContainer, WidgetId};
use std::ops::Deref;
use std::time::Instant;

const INPUT_DELAY: u128 = 200;

#[repr(u8)]
pub enum ContainerAlignment {
    Left,
    Right,
    Top,
    Bottom,
}

pub trait WindowState {
    fn on_revert_focus(&mut self);
}

/// Container for our widgets that lays them out in the tree
/// Has higher level management methods to perfrom things
/// on the entire UI tree
pub struct WidgetTree<T: WindowState, D> {
    pub root: WidgetContainer<T, D>,
    pub focus: Vec<WidgetId>,
    pub data: T,
    last_focused: Instant,
}
impl<T: WindowState, D> WidgetTree<T, D> {
    pub fn new(root: WidgetContainer<T, D>, data: T, initial_focus: WidgetId) -> WidgetTree<T, D> {
        WidgetTree::<T, D> { root, focus: vec![initial_focus], data, last_focused: Instant::now() }
    }

    pub fn render_all(&mut self, graphics: &mut crate::Graphics) {
        let focused_widget = *self.focus.last().unwrap();
        self.root.widget.set_layout(graphics.bounds);
        self.root.render(&self.data, graphics, &focused_widget);
    }

    fn current_focus<'a>(
        focus: &'a [WidgetId],
        root: &'a mut WidgetContainer<T, D>,
    ) -> Option<&'a mut WidgetContainer<T, D>> {
        if let Some(last) = focus.last() {
            return root.find_widget_mut(*last);
        }
        None
    }

    pub fn focus(&mut self, widget: WidgetId, animations: &mut AnimationManager) {
        //Find current focus so we can notify it is about to lose
        if let Some(lost) = Self::current_focus(&self.focus, &mut self.root) {
            lost.widget.lost_focus(&self.data, animations);
            self.last_focused = Instant::now();
        }

        //Find new focus
        if let Some(got) = self.root.find_widget_mut(widget) {
            got.widget.got_focus(&self.data, animations);
            self.focus.push(widget);
        }
    }

    pub fn revert_focus(&mut self, animations: &mut AnimationManager) {
        let now = Instant::now();

        //Check if we have pressed back multiple times in quick succession
        //If we have revert all the way to the last different widget
        //This will allow us to get back to the platform list after going deep in a plugin
        //items
        self.data.on_revert_focus();

        let mut last = self.focus.pop();
        if (now - self.last_focused).as_millis() < INPUT_DELAY {
            while last.as_ref() == self.focus.last() {
                last = self.focus.pop();
            }
        }
        self.last_focused = now;

        //Find current focus so we can notify it is about to lose
        if let Some(last) = last {
            if let Some(lost) = self.root.find_widget_mut(last) {
                lost.widget.lost_focus(&self.data, animations);
            }
        }

        //Revert to previous focus
        if let Some(got) = Self::current_focus(&self.focus, &mut self.root) {
            got.widget.got_focus(&self.data, animations);
        }
    }
}

impl<T: WindowState, D> Deref for WidgetTree<T, D> {
    type Target = WidgetContainer<T, D>;
    fn deref(&self) -> &Self::Target { &self.root }
}
