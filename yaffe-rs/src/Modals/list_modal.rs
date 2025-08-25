use crate::ui::{AnimationManager, ContainerSize, List, ListItem, UiContainer, UiElement, WidgetId};
use crate::modals::ModalAction;
use crate::{Actions, Graphics, LogicalSize};

crate::widget!(
    pub struct ListModal {
        container: UiContainer<(), ModalAction> = UiContainer::column()
    }
);

impl ListModal {
    pub fn from<T: ListItem + 'static>(items: Vec<T>) -> ListModal {
        let mut modal = ListModal::new();
        let list = List::<T>::from(items);
        modal.container.add_child(list, ContainerSize::Shrink);
        modal
    }

    pub fn get_selected<T: ListItem + 'static>(&self) -> &T {
        let element = &self.container.get_child(0);
        element.as_any().downcast_ref::<List<T>>().unwrap().get_selected()
    }
}

impl UiElement<(), ModalAction> for ListModal {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize { self.container.calc_size(graphics) }

    fn action(
        &mut self,
        state: &mut (),
        animations: &mut AnimationManager,
        action: &Actions,
        handler: &mut ModalAction,
    ) -> bool {
        handler.close_if_accept(action) || self.container.action(state, animations, action, handler)
    }

    fn render(&mut self, graphics: &mut Graphics, state: &(), current_focus: &WidgetId) {
        self.container.render(graphics, state, current_focus);
    }
}
