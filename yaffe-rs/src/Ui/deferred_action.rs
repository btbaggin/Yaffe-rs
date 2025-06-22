use crate::ui::{display_modal, AnimationManager, FocusType, WidgetId, WidgetTree};
use crate::state::YaffeState;

pub struct DeferredAction {
    focus: Option<FocusType>,
    load_plugin: bool,
    message: Option<String>,
}
// TODO make this a type parameter on widget tree?
impl DeferredAction {
    pub fn new() -> DeferredAction { DeferredAction { focus: None, load_plugin: false, message: None } }
    pub fn focus_widget<T: 'static>(&mut self) { self.focus = Some(FocusType::Focus(WidgetId::of::<T>())); }
    pub fn revert_focus(&mut self) { self.focus = Some(FocusType::Revert); }
    pub fn load_plugin(&mut self) { self.load_plugin = true; }
    pub fn display_message(&mut self, message: String) { self.message = Some(message); }

    pub fn resolve(self, ui: &mut WidgetTree<YaffeState>, animations: &mut AnimationManager) {
        match self.focus {
            None => { /*do nothing*/ }
            Some(FocusType::Revert) => ui.revert_focus(animations),
            Some(FocusType::Focus(w)) => ui.focus(w, animations),
        }

        if self.load_plugin {
            let state = &mut ui.data;
            if let crate::state::GroupType::Plugin(index) = state.get_selected_group().kind {
                crate::plugins::load_plugin_items(state, index);
            }
        }

        if let Some(message) = self.message {
            let message = Box::new(crate::modals::MessageModalContent::new(&message));
            display_modal(&mut ui.data, "Error", None, message, None);
        }
    }
}
