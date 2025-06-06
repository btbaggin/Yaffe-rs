use crate::ui::{FocusType, WidgetTree, WidgetId};

pub struct DeferredAction {
    focus: Option<FocusType>,
    load_plugin: bool,
    message: Option<String>,
}
impl DeferredAction {
    pub fn new() -> DeferredAction {
        DeferredAction { 
            focus: None,
            load_plugin: false,
            message: None,
        }
    }
    pub fn focus_widget(&mut self, widget: WidgetId) {
        self.focus = Some(FocusType::Focus(widget));
    }
    pub fn revert_focus(&mut self) {
        self.focus = Some(FocusType::Revert);
    }
    pub fn load_plugin(&mut self) {
        self.load_plugin = true;
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

        if self.load_plugin {
            let state = &mut ui.data;
            if let crate::state::GroupType::Plugin(index) =  state.groups[state.selected.group_index].kind {
                crate::plugins::load_plugin_items(state, index);
            }
        }

        if let Some(message) = self.message {
            let message = Box::new(crate::modals::MessageModalContent::new(&message));
            crate::ui::display_modal(&mut ui.data, "Error", None, message, None);
        }
    }
}