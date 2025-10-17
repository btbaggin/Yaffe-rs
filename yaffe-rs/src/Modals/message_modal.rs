use crate::controls::Label;
use crate::modals::{ModalContentElement, ModalInputHandler};
use crate::ui::{ContainerSize, UiContainer};
use crate::DeferredAction;
use std::marker::PhantomData;

pub struct MessageModal<T> {
    _data: PhantomData<T>,
}

impl<T: 'static> MessageModal<T> {
    pub fn from(message: &str) -> ModalContentElement<T> {
        let mut modal = ModalContentElement::new(MessageModal { _data: PhantomData }, false);
        modal.add_child(Label::wrapping(message, None), ContainerSize::Shrink);
        modal
    }
}
impl<T: 'static> ModalInputHandler<T> for MessageModal<T> {
    fn as_any(&self) -> &dyn std::any::Any { self }

    fn on_close(&self, _: &mut T, _: bool, _: &UiContainer<T>, _: &mut DeferredAction<T>) {}
}
