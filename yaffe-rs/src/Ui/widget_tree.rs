use crate::input::Actions;
use crate::modals::{update_modal, Modal, Toast};
use crate::ui::{AnimationManager, DeferredAction, LayoutElement, UiContainer, UiElement, WidgetId};
use std::ops::Deref;
use std::sync::Mutex;
use std::time::Instant;

/// Container for our widgets that lays them out in the tree
/// Has higher level management methods to perfrom things
/// on the entire UI tree
pub struct WidgetTree<T: 'static> {
    pub root: UiContainer<T>,
    pub focus: Vec<WidgetId>,
    pub data: T,
    pub animations: AnimationManager,
    pub modals: Mutex<Vec<Modal<T>>>, //TODO make private?
    pub toasts: Vec<Toast>,
    last_focused: Instant,
}
impl<T> WidgetTree<T> {
    pub fn new(root: UiContainer<T>, data: T, initial_focus: WidgetId) -> WidgetTree<T> {
        WidgetTree::<T> {
            root,
            focus: vec![initial_focus],
            data,
            animations: AnimationManager::new(),
            modals: Mutex::new(vec![]),
            toasts: vec![],
            last_focused: Instant::now(),
        }
    }

    pub fn render(&mut self, graphics: &mut crate::Graphics) {
        let focused_widget = *self.focus.last().unwrap();
        let old_bounds = graphics.bounds;
        self.root.set_layout(graphics.bounds);
        self.root.render(graphics, &self.data, &focused_widget);

        if !self.toasts.is_empty() {
            // Render calls will modify the bounds, so we must reset it
            graphics.bounds = old_bounds;
            crate::modals::render_toasts(&self.toasts, graphics);
        }

        //Render modal last, on top of everything
        let modals = &mut self.modals.lock().unwrap();
        if let Some(m) = modals.last_mut() {
            // Render calls will modify the bounds, so we must reset it
            graphics.bounds = old_bounds;
            crate::modals::render_modal(m, &mut self.data, graphics);
        }
    }

    pub fn action(&mut self, action: &Actions, handler: &mut DeferredAction<T>) -> bool {
        //This method can call into display_modal above, which locks the mutex
        //If we lock here that call will wait infinitely
        //We can get_mut here to ensure compile time exclusivity instead of locking
        //That allows us to call display_modal in close() below
        if update_modal(self, action, handler) {
            true
        } else {
            let focus = self.focus.last().unwrap();
            if let Some(e) = self.root.find_widget_mut(*focus) {
                e.action(&mut self.data, &mut self.animations, action, handler)
            } else {
                crate::logger::warn!("Unable to find focused element");
                false
            }
        }
    }

    pub fn fixed_update(&mut self, delta_time: f32) -> bool {
        let has_toasts = !self.toasts.is_empty();
        let has_animations = self.animations.is_dirty();

        self.animations.process(&mut self.root, &mut self.modals, delta_time);
        Toast::process_toast(&mut self.toasts, delta_time);
        has_animations || has_toasts
    }

    pub fn display_toast(&mut self, toast: Toast) { self.toasts.push(toast); }

    pub fn is_modal_open(&self) -> bool {
        let modals = self.modals.lock().unwrap();
        !modals.is_empty()
    }

    fn current_focus<'a>(focus: &'a [WidgetId], root: &'a mut UiContainer<T>) -> Option<&'a mut dyn UiElement<T>> {
        if let Some(last) = focus.last() {
            return root.find_widget_mut(*last);
        }
        None
    }

    pub fn focus(&mut self, widget: WidgetId) {
        //Find current focus so we can notify it is about to lose
        if let Some(lost) = Self::current_focus(&self.focus, &mut self.root) {
            lost.lost_focus(&self.data, &mut self.animations);
            self.last_focused = Instant::now();
        }

        //Find new focus
        if let Some(got) = self.root.find_widget_mut(widget) {
            got.got_focus(&self.data, &mut self.animations);
            self.focus.push(widget);
        }
    }

    pub fn revert_focus(&mut self) {
        let now = Instant::now();

        let mut last = self.focus.pop();
        while last.as_ref() == self.focus.last() {
            last = self.focus.pop();
        }
        self.last_focused = now;

        //Find current focus so we can notify it is about to lose
        if let Some(last) = last {
            if let Some(lost) = self.root.find_widget_mut(last) {
                lost.lost_focus(&self.data, &mut self.animations);
            }
        }

        //Revert to previous focus
        if let Some(got) = Self::current_focus(&self.focus, &mut self.root) {
            got.got_focus(&self.data, &mut self.animations);
        }
    }
}

impl<T> Deref for WidgetTree<T> {
    type Target = UiContainer<T>;
    fn deref(&self) -> &Self::Target { &self.root }
}
