use crate::input::Actions;
use crate::modals::{Modal, update_modal};
use crate::ui::{AnimationManager, LayoutElement, UiContainer, UiElement, WidgetId};
use std::ops::Deref;
use std::time::Instant;
use std::sync::Mutex;

/// Container for our widgets that lays them out in the tree
/// Has higher level management methods to perfrom things
/// on the entire UI tree
pub struct WidgetTree<T: 'static, D: 'static> {
    pub root: UiContainer<T, D>,
    pub focus: Vec<WidgetId>,
    pub data: T,
    pub modals: Mutex<Vec<Modal<T, D>>>,
    last_focused: Instant,
}
impl<T, D> WidgetTree<T, D> {
    pub fn new(root: UiContainer<T, D>, data: T, initial_focus: WidgetId) -> WidgetTree<T, D> {
        WidgetTree::<T, D> { root, focus: vec![initial_focus], data, modals: Mutex::new(vec!()), last_focused: Instant::now() }
    }

    pub fn render(&mut self, graphics: &mut crate::Graphics) {
        let focused_widget = *self.focus.last().unwrap();
        let old_bounds = graphics.bounds;
        self.root.set_layout(graphics.bounds);
        self.root.render(graphics, &self.data, &focused_widget);

        //Render modal last, on top of everything
        let modals = &mut self.modals.lock().unwrap();
        if let Some(m) = modals.last_mut() {
            // Render calls will modify the bounds, so we must reset it
            graphics.bounds = old_bounds;
            crate::modals::render_modal(m, graphics);
        }
    }

    pub fn action(&mut self, animations: &mut AnimationManager, action: &Actions, handler: &mut D) -> bool {
        //This method can call into display_modal above, which locks the mutex
        //If we lock here that call will wait infinitely
        //We can get_mut here to ensure compile time exclusivity instead of locking
        //That allows us to call display_modal in close() below
        if update_modal(&mut self.modals, &mut self.data, animations, action, handler) {
            true
        } else {
            let focus = self.focus.last().unwrap();
            if let Some(e) = self.root.find_widget_mut(*focus) {
                e.action(&mut self.data, animations, action, handler)
            } else {
                crate::logger::warn!("Unable to find focused element");
                false
            }
        }
    }

    pub fn is_modal_open(&self) -> bool {
        let modals = self.modals.lock().unwrap();
        !modals.is_empty()
    }

    fn current_focus<'a>(
        focus: &'a [WidgetId],
        root: &'a mut UiContainer<T, D>,
    ) -> Option<&'a mut dyn UiElement<T, D>> {
        if let Some(last) = focus.last() {
            return root.find_widget_mut(*last);
        }
        None
    }

    pub fn focus(&mut self, widget: WidgetId, animations: &mut AnimationManager) {
        //Find current focus so we can notify it is about to lose
        if let Some(lost) = Self::current_focus(&self.focus, &mut self.root) {
            lost.lost_focus(&self.data, animations);
            self.last_focused = Instant::now();
        }

        //Find new focus
        if let Some(got) = self.root.find_widget_mut(widget) {
            got.got_focus(&self.data, animations);
            self.focus.push(widget);
        }
    }

    pub fn revert_focus(&mut self, animations: &mut AnimationManager) {
        let now = Instant::now();

        let mut last = self.focus.pop();
        while last.as_ref() == self.focus.last() {
            last = self.focus.pop();
        }
        self.last_focused = now;

        //Find current focus so we can notify it is about to lose
        if let Some(last) = last {
            if let Some(lost) = self.root.find_widget_mut(last) {
                lost.lost_focus(&self.data, animations);
            }
        }

        //Revert to previous focus
        if let Some(got) = Self::current_focus(&self.focus, &mut self.root) {
            got.got_focus(&self.data, animations);
        }
    }
}

impl<T, D> Deref for WidgetTree<T, D> {
    type Target = UiContainer<T, D>;
    fn deref(&self) -> &Self::Target { &self.root }
}
