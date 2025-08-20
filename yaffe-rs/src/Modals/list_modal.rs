use crate::ui::{List, ListItem, UiElement, WidgetId, ModalAction, AnimationManager, UiContainer, ContainerSize, LayoutElement};
use crate::{Actions, Graphics};

// Allow displaying a list of items that can be selected
// Items must implement `ListItem` trait
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
    fn action(&mut self, state: &mut (), animations: &mut AnimationManager, action: &Actions, handler: &mut ModalAction) -> bool {
        handler.close_if_accept(action) || self.container.action(state, animations, action, handler)
    }

    fn render(&mut self, graphics: &mut Graphics, state: &(), current_focus: &WidgetId) {
        // TODO this is soooooo fucked
        self.container.render(graphics, state, current_focus);
        self.set_layout(self.container.layout());
    }
}
