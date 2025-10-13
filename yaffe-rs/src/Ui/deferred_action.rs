use crate::modals::{MessageModal, ModalSize, ModalDisplay, display_modal_raw};
use crate::state::{GroupType, YaffeState};
use crate::ui::{AnimationManager, FocusType, WidgetId, WidgetTree};

// Core trait for deferred actions
pub trait DeferredActionTrait {
    fn resolve(self: Box<Self>, ui: &mut WidgetTree<YaffeState, DeferredAction>, animations: &mut AnimationManager) -> Option<DeferredAction>;
}

// Container that holds all deferred actions
pub struct DeferredAction {
    actions: Vec<Box<dyn DeferredActionTrait>>,
}

impl DeferredAction {
    pub fn new() -> Self {
        Self {
            actions: Vec::with_capacity(4),
        }
    }

    // Builder-style methods that capture data at call time
    pub fn focus_widget(&mut self, id: WidgetId) {
        self.actions.push(Box::new(FocusWidgetAction { id }));
    }

    pub fn revert_focus(&mut self) {
        self.actions.push(Box::new(RevertFocusAction));
    }

    pub fn load_plugin(&mut self, reset_items: bool) {
        self.actions.push(Box::new(LoadPluginAction { reset_items }));
    }

    pub fn display_message(&mut self, message: String) {
        self.actions.push(Box::new(DisplayMessageAction { message }));
    }

    pub fn display_modal(&mut self, modal: ModalDisplay<YaffeState, DeferredAction>) {
        self.actions.push(Box::new(DisplayModalAction { modal }));
    }

    pub fn resolve(self, ui: &mut WidgetTree<YaffeState, DeferredAction>, animations: &mut AnimationManager) {
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

impl DeferredActionTrait for FocusWidgetAction {
    fn resolve(self: Box<Self>, ui: &mut WidgetTree<YaffeState, DeferredAction>, animations: &mut AnimationManager) -> Option<DeferredAction> {
        ui.focus(self.id, animations);
        
        // Example: focusing this widget also displays a message
        let mut new_actions = DeferredAction::new();
        new_actions.display_message("Widget focused!".to_string());
        Some(new_actions)
    }
}

struct RevertFocusAction;

impl DeferredActionTrait for RevertFocusAction {
    fn resolve(self: Box<Self>, ui: &mut WidgetTree<YaffeState, DeferredAction>, animations: &mut AnimationManager) -> Option<DeferredAction> {
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
            ui.revert_focus(animations)
        }
        None
    }
}

struct LoadPluginAction {
    reset_items: bool,
}

impl DeferredActionTrait for LoadPluginAction {
    fn resolve(self: Box<Self>, ui: &mut WidgetTree<YaffeState, DeferredAction>, _animations: &mut AnimationManager) -> Option<DeferredAction> {
        let state = &mut ui.data;
        let group = &mut state.groups[state.selected.group_index()];
        if let GroupType::Plugin(index) = group.kind {
            if self.reset_items {
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

impl DeferredActionTrait for DisplayMessageAction {
    fn resolve(self: Box<Self>, ui: &mut WidgetTree<YaffeState, DeferredAction>, _animations: &mut AnimationManager) -> Option<DeferredAction> {
        display_error(ui, self.message);
        None
    }
}

struct DisplayModalAction {
    modal: ModalDisplay<YaffeState, DeferredAction>,
}

impl DeferredActionTrait for DisplayModalAction {
    fn resolve(self: Box<Self>, ui: &mut WidgetTree<YaffeState, DeferredAction>, _animations: &mut AnimationManager) -> Option<DeferredAction> {
        self.modal.display(ui);
        None
    }
}

fn display_error(ui: &mut WidgetTree<YaffeState, DeferredAction>, message: String) {
    let message = MessageModal::from(&message);
    display_modal_raw(ui, "Error", None, message, ModalSize::Half, None);
}