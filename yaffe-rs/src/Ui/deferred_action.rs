use crate::modals::{MessageModal, ModalSize, ModalDisplay, display_modal_raw};
use crate::state::{GroupType, YaffeState};
use crate::ui::{AnimationManager, FocusType, WidgetId, WidgetTree};

pub struct DeferredAction {
    focus: Option<FocusType>,
    load_plugin: Option<bool>,
    message: Option<String>,
    modals: Vec<ModalDisplay<YaffeState, DeferredAction>>
}
impl DeferredAction {
    pub fn new() -> DeferredAction { DeferredAction { focus: None, load_plugin: None, message: None, modals: Vec::with_capacity(1) } }
    pub fn focus_widget(&mut self, id: WidgetId) { self.focus = Some(FocusType::Focus(id)); }
    pub fn revert_focus(&mut self) { self.focus = Some(FocusType::Revert); }
    pub fn load_plugin(&mut self, reset_items: bool) { self.load_plugin = Some(reset_items); }
    pub fn display_message(&mut self, message: String) { self.message = Some(message); }
    pub fn display_modal(&mut self, modal: ModalDisplay<YaffeState, DeferredAction>) { self.modals.push(modal); }

    pub fn resolve(self, ui: &mut WidgetTree<YaffeState, DeferredAction>, animations: &mut AnimationManager) {
        match self.focus {
            None => { /*do nothing*/ }
            Some(FocusType::Revert) => {
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
            },
            Some(FocusType::Focus(w)) => ui.focus(w, animations),
        }

        if let Some(reset) = self.load_plugin {
            let state = &mut ui.data;
            let group = &mut state.groups[state.selected.group_index()];
            if let GroupType::Plugin(index) = group.kind {
                if reset {
                    group.tiles.clear();
                }
                if let Err(e) = crate::plugins::load_plugin_items(state, index) {
                    display_error(ui, format!("Error loading plugin items: {e}"));
                }
            }
        }

        if let Some(message) = self.message {
            display_error(ui, message)
        }

        for m in self.modals {
            m.display(ui)
        }
    }
}

fn display_error(ui: &mut WidgetTree<YaffeState, DeferredAction>, message: String) {
     let message = MessageModal::from(&message);
    display_modal_raw(ui, "Error", None, message, ModalSize::Half, None);
}
