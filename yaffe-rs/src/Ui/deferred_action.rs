use crate::modals::{display_error, DisplayModal, Toast};
use crate::state::{GroupType, YaffeState};
use crate::ui::{AnimationManager, WidgetId, WidgetTree};

pub trait DeferredActionTrait<T> {
    fn resolve(self: Box<Self>, ui: &mut WidgetTree<T>, animations: &mut AnimationManager)
        -> Option<DeferredAction<T>>;
}

pub struct DeferredAction<T> {
    actions: Vec<Box<dyn DeferredActionTrait<T>>>,
}

impl<T: 'static> DeferredAction<T> {
    pub fn new() -> Self { Self { actions: Vec::with_capacity(4) } }

    pub fn defer(&mut self, action: impl DeferredActionTrait<T> + 'static) { self.actions.push(Box::new(action)); }

    pub fn focus_widget(&mut self, id: WidgetId) { self.actions.push(Box::new(FocusWidgetAction { id })); }

    pub fn display_message(&mut self, message: String) {
        self.actions.push(Box::new(DisplayMessageAction { message }));
    }

    pub fn display_modal(&mut self, modal: DisplayModal<T>) {
        self.actions.push(Box::new(modal));
    }

    pub fn display_toast(&mut self, message: &str, time: f32) {
        self.actions.push(Box::new(Toast::new(message, time)))
    }

    pub fn resolve(self, ui: &mut WidgetTree<T>, animations: &mut AnimationManager) {
        let mut queue = self.actions;

        while let Some(action) = queue.pop() {
            if let Some(mut new_actions) = action.resolve(ui, animations) {
                // Prepend new actions to process them next
                queue.append(&mut new_actions.actions);
            }
        }
    }
}

// Individual action types - each captures its own data
struct FocusWidgetAction {
    id: WidgetId,
}

impl<T> DeferredActionTrait<T> for FocusWidgetAction {
    fn resolve(
        self: Box<Self>,
        ui: &mut WidgetTree<T>,
        animations: &mut AnimationManager,
    ) -> Option<DeferredAction<T>> {
        ui.focus(self.id, animations);
        None
    }
}

pub struct RevertFocusAction;

impl DeferredActionTrait<YaffeState> for RevertFocusAction {
    // TODO can focus move to widgettree?
    fn resolve(
        self: Box<Self>,
        ui: &mut WidgetTree<YaffeState>,
        animations: &mut AnimationManager,
    ) -> Option<DeferredAction<YaffeState>> {
        let state = &mut ui.data;
        if state.navigation_stack.borrow_mut().pop().is_some() {
            state.selected.tile_index = 0;
            let group = &mut state.groups[state.selected.group_index()];
            group.tiles.clear();
            if let crate::state::GroupType::Plugin(index) = group.kind {
                if let Err(e) = crate::plugins::load_plugin_items(state, index) {
                    display_error(ui, format!("Error loading plugin items: {e}"));
                }
            }
        } else {
            ui.revert_focus(animations);
        }
        None
    }
}

pub struct LoadPluginAction(pub bool);

impl DeferredActionTrait<YaffeState> for LoadPluginAction {
    fn resolve(
        self: Box<Self>,
        ui: &mut WidgetTree<YaffeState>,
        _animations: &mut AnimationManager,
    ) -> Option<DeferredAction<YaffeState>> {
        let state = &mut ui.data;
        let group = &mut state.groups[state.selected.group_index()];
        if let GroupType::Plugin(index) = group.kind {
            if self.0 {
                group.tiles.clear();
            }
            if let Err(e) = crate::plugins::load_plugin_items(state, index) {
                display_error(ui, format!("Error loading plugin items: {e}"));
            }
        }
        None
    }
}

struct DisplayMessageAction {
    message: String,
}

impl<T> DeferredActionTrait<T> for DisplayMessageAction {
    fn resolve(
        self: Box<Self>,
        ui: &mut WidgetTree<T>,
        _animations: &mut AnimationManager,
    ) -> Option<DeferredAction<T>> {
        display_error(ui, self.message);
        None
    }
}
