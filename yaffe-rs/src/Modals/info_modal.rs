use crate::ui::{UiElement, WidgetId, ModalAction, AnimationManager, UiContainer, ContainerSize};
use crate::widgets::{InfoPane};
use crate::{Actions, Graphics, LogicalSize, Tile};

crate::widget!(
    pub struct InfoModal {
        container: UiContainer<(), ModalAction> = UiContainer::column()
    }
);

impl InfoModal {
    pub fn from(items: &Tile) -> InfoModal { 
        let mut modal = InfoModal::new();
        let mut attributes = vec!();
        for (name, value) in &items.metadata {
            attributes.push((name.clone(), value.clone()))
        }
        let pane = InfoPane::from(items.boxart.clone(), items.description.clone(), attributes);
        modal.container
            .with_child(UiContainer::row(), ContainerSize::Percent(0.60))
                .add_child(pane, ContainerSize::Fill);
        modal
    }
}

impl UiElement<(), ModalAction> for InfoModal {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize {
        self.container.calc_size(graphics)
    }

    fn action(&mut self, state: &mut (), animations: &mut AnimationManager, action: &Actions, handler: &mut ModalAction) -> bool {
        handler.close_if_accept(action) || self.container.action(state, animations, action, handler)
    }

    fn render(&mut self, graphics: &mut Graphics, state: &(), current_focus: &WidgetId) {
        self.container.render(graphics, state, current_focus);
    }
}
