use super::{display_modal_raw, ModalContentElement, ModalSize, Toast};
use crate::ui::{DeferredAction, DeferredActionTrait, WidgetTree};
use std::marker::PhantomData;

pub struct ModalClose<T> {
    close: bool,
    _data: PhantomData<T>,
}
impl<T> DeferredActionTrait<T> for ModalClose<T> {
    fn resolve(self: Box<Self>, ui: &mut WidgetTree<T>) -> Option<DeferredAction<T>> {
        let modals = ui.modals.get_mut().unwrap();
        let modal = modals.last().unwrap();
        // Content will always be second (after title, before buttons)
        let content = crate::convert_to!(modal.content.get_child(1).as_ref(), ModalContentElement<T>);
        if content.handler.validate(self.close, content) {
            let mut new_actions = DeferredAction::new();
            content.handler.on_close(&mut ui.data, self.close, content, &mut new_actions);
            modals.pop();
            return Some(new_actions);
        }
        None
    }
}
impl<T: 'static> ModalClose<T> {
    pub fn close_if_accept(action: &crate::Actions, handler: &mut DeferredAction<T>) -> bool {
        match action {
            crate::Actions::Accept => {
                handler.defer(ModalClose { close: true, _data: PhantomData });
                true
            }
            crate::Actions::Back => {
                handler.defer(ModalClose { close: false, _data: PhantomData });
                true
            }
            _ => false,
        }
    }
}

pub struct DisplayModal<T: 'static> {
    title: String,
    confirmation_button: Option<String>,
    content: ModalContentElement<T>,
    width: ModalSize,
}
impl<T> DisplayModal<T> {
    pub fn new(
        title: &str,
        confirmation_button: Option<&str>,
        content: ModalContentElement<T>,
        width: ModalSize,
    ) -> DisplayModal<T> {
        DisplayModal {
            title: String::from(title),
            confirmation_button: confirmation_button.map(String::from),
            content,
            width,
        }
    }
}

impl<T> DeferredActionTrait<T> for DisplayModal<T> {
    fn resolve(self: Box<Self>, ui: &mut WidgetTree<T>) -> Option<DeferredAction<T>> {
        display_modal_raw(ui, &self.title, self.confirmation_button.as_deref(), self.content, self.width);
        None
    }
}

impl<T> DeferredActionTrait<T> for Toast {
    fn resolve(self: Box<Self>, ui: &mut WidgetTree<T>) -> Option<DeferredAction<T>> {
        ui.display_toast(*self);
        None
    }
}
