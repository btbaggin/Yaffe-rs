use std::marker::PhantomData;
use crate::ui::{WidgetTree, AnimationManager, DeferredAction, DeferredActionTrait};
use super::{ModalContentElement, ModalOnClose, ModalSize, display_modal_raw, Toast};

pub struct ModalClose<T> {
    close: bool,
    _data: PhantomData<T>,
}
impl<T> DeferredActionTrait<T> for ModalClose<T> {
    fn resolve(
        self: Box<Self>,
        ui: &mut WidgetTree<T>,
        _animations: &mut AnimationManager,
    ) -> Option<DeferredAction<T>> {
        let modals = ui.modals.get_mut().unwrap();
        let modal = modals.pop().unwrap();
        if let Some(close) = modal.on_close {
            // Content will always be second (after title, before buttons)
            let content = crate::convert_to!(modal.content.get_child(1).as_ref(), ModalContentElement<T>);

            let mut new_actions = DeferredAction::new();
            close(&mut ui.data, self.close, content, &mut new_actions);
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
    on_close: Option<ModalOnClose<T>>,
}
impl<T> DisplayModal<T> {
    pub fn new(
        title: &str,
        confirmation_button: Option<&str>,
        content: ModalContentElement<T>,
        width: ModalSize,
        on_close: Option<ModalOnClose<T>>,
    ) -> DisplayModal<T> {
        DisplayModal {
            title: String::from(title),
            confirmation_button: confirmation_button.map(String::from),
            content,
            width,
            on_close,
        }
    }
    pub fn display(self, ui: &mut WidgetTree<T>) {
        display_modal_raw(ui, &self.title, self.confirmation_button.as_deref(), self.content, self.width, self.on_close)
    }
}

impl<T> DeferredActionTrait<T> for DisplayModal<T> {
    fn resolve(
        self: Box<Self>,
        ui: &mut WidgetTree<T>,
        _animations: &mut AnimationManager,
    ) -> Option<DeferredAction<T>> {
        self.display(ui);
        None
    }
}


impl<T> DeferredActionTrait<T> for Toast {
    fn resolve(
        self: Box<Self>,
        ui: &mut WidgetTree<T>,
        _animations: &mut AnimationManager,
    ) -> Option<DeferredAction<T>> {
        // TODO
        ui.display_toast(*self);
        None
    }
}