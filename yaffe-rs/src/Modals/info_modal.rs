use crate::modals::{ModalContentElement, ModalInputHandler};
use crate::ui::{ContainerSize, UiContainer};
use crate::widgets::InfoPane;
use crate::{DeferredAction, Tile, YaffeState};

pub struct InfoModal;

impl InfoModal {
    pub fn from(items: &Tile) -> ModalContentElement<YaffeState> {
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

impl ModalInputHandler<YaffeState> for InfoModal {
    fn as_any(&self) -> &dyn std::any::Any { self }

    fn on_close(&self, _: &mut YaffeState, _: bool, _: &UiContainer<YaffeState>, _: &mut DeferredAction<YaffeState>) {}
}
