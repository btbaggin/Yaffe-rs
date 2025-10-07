use crate::modals::{display_modal, MessageModal, ModalSize};
use crate::state::{GroupType, YaffeState};
use crate::ui::{AnimationManager, FocusType, WidgetId, WidgetTree};

pub struct DeferredAction {
    focus: Option<FocusType>,
    load_plugin: Option<bool>,
    message: Option<String>,
}
impl DeferredAction {
    pub fn new() -> DeferredAction { DeferredAction { focus: None, load_plugin: None, message: None } }
    pub fn focus_widget(&mut self, id: WidgetId) { self.focus = Some(FocusType::Focus(id)); }
    pub fn revert_focus(&mut self) { self.focus = Some(FocusType::Revert); }
    pub fn load_plugin(&mut self, reset_items: bool) { self.load_plugin = Some(reset_items); }
    pub fn display_message(&mut self, message: String) { self.message = Some(message); }

    pub fn resolve(self, ui: &mut WidgetTree<YaffeState, DeferredAction>, animations: &mut AnimationManager) {
        match self.focus {
            None => { /*do nothing*/ }
            Some(FocusType::Revert) => ui.revert_focus(animations),
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
                    let message = format!("Error loading plugin items: {e:?}");
                    let message = MessageModal::from(&message);
                    display_modal(&mut ui.data, "Error", None, message, ModalSize::Half, None);
                }
            }
        }

        if let Some(message) = self.message {
            let message = MessageModal::from(&message);
            display_modal(&mut ui.data, "Error", None, message, ModalSize::Half, None);
        }
    }
}
