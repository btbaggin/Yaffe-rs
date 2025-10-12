use crate::modals::{ModalInputHandler, ModalContentElement};
use crate::ui::{ContainerSize, UiContainer};
use crate::widgets::InfoPane;
use crate::Tile;

pub struct InfoModal;

impl InfoModal {
    pub fn from(items: &Tile) -> ModalContentElement {
        let mut attributes = vec![];
        for (name, value) in &items.metadata {
            attributes.push((name.clone(), value.clone()))
        }
        let pane = InfoPane::from(items.boxart.clone(), items.description.clone(), attributes);

        let mut modal = ModalContentElement::new(InfoModal, false);
        modal.with_child(UiContainer::row(), ContainerSize::Percent(0.60)).add_child(pane, ContainerSize::Fill);
        modal
    }
}

impl ModalInputHandler for InfoModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
}
