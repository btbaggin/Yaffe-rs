use crate::controls::Label;
use crate::modals::{ModalInputHandler, ModalContentElement};
use crate::ui::ContainerSize;

pub struct MessageModal;

impl MessageModal {
    pub fn from(message: &str) -> ModalContentElement {
        let mut modal = ModalContentElement::new(MessageModal, false);
        modal.add_child(Label::wrapping(message, None), ContainerSize::Shrink);
        modal
    }
}
impl ModalInputHandler for MessageModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
}
