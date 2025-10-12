use crate::controls::{List, ListItem};
use crate::modals::{ModalInputHandler, ModalContentElement};
use crate::ui::ContainerSize;

pub struct ListModal;

impl ListModal {
    pub fn from<T: ListItem + 'static>(items: Vec<T>) -> ModalContentElement {
        let mut modal = ModalContentElement::new(ListModal, false);
        let list = List::<T>::from(items);
        modal.add_child(list, ContainerSize::Shrink);
        modal
    }
}

impl ModalInputHandler for ListModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
}
