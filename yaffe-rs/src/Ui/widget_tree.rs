use crate::YaffeState;
use std::ops::Deref;
use std::time::Instant;
use std::rc::Rc;
use std::cell::RefCell;
use crate::ui::{WidgetContainer, WidgetId, AnimationManager};     

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
    animations: Rc<RefCell<AnimationManager>>,
    last_focus: (Option<WidgetId>, Instant),
}
impl WidgetTree {
    pub fn new(root: WidgetContainer, animations: Rc<RefCell<AnimationManager>>, data: YaffeState) -> WidgetTree {
        WidgetTree {
            root,
            focus: vec!(),
            data,
            animations,
            last_focus: (None, Instant::now()),
        }
    }

    pub fn render_all(&mut self, graphics: &mut crate::Graphics) {
        self.root.widget.set_layout(graphics.bounds);
        self.root.render(&self.data, graphics);
    }

    pub fn needs_new_frame(&self) -> bool {
        let a = self.animations.borrow();
        a.is_dirty()
    }

    pub fn process_animations(&mut self, delta_time: f32) {
        let mut animator = self.animations.borrow_mut();
        animator.process(&mut self.root, delta_time)
    }

    fn current_focus<'a>(focus: &'a Vec<WidgetId>, root: &'a mut WidgetContainer) -> Option<&'a mut WidgetContainer> {
        if let Some(last) = focus.last() {
            return root.find_widget_mut(*last);
        }
        None
    }

    pub fn focus(&mut self, widget: WidgetId) {
        //Find current focus so we can notify it is about to lose
        if let Some(lost) = Self::current_focus(&self.focus, &mut self.root) {
            lost.widget.lost_focus(&self.data);
            self.last_focus = (Some(lost.widget.get_id()), Instant::now());
        }
    
        //Find new focus
        if let Some(got) = self.root.find_widget_mut(widget) {
            got.widget.got_focus(&self.data);
            self.focus.push(widget);
        }
    }

    pub fn revert_focus(&mut self) {
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
        
        //Find current focus so we can notify it is about to lose
        if let Some(last) = last {
            if let Some(lost) = self.root.find_widget_mut(last) {
                lost.widget.lost_focus(&self.data);
            }
        }

        //Revert to previous focus
        if let Some(got) = Self::current_focus(&self.focus, &mut self.root) {
            got.widget.got_focus(&self.data);
        }

        if !different {
            //The only scenario this could happen is plugins
            let state = &mut self.data;
            crate::plugins::load_plugin_items(crate::plugins::NavigationAction::Back, state);
        }

    }
}

impl Deref for WidgetTree {
    type Target = WidgetContainer;
    fn deref(&self) -> &Self::Target {
        &self.root
    }
}