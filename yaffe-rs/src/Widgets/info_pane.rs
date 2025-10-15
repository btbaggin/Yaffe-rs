use crate::assets::AssetKey;
use crate::controls::{Image, Label};
use crate::ui::{AnimationManager, ContainerSize, DeferredAction, UiContainer, UiElement, WidgetId};
use crate::{widget, Actions, Graphics, LogicalSize};

widget!(
    pub struct InfoPane<T> {
        container: UiContainer<T> = UiContainer::column()
    }
);
impl<T> InfoPane<T> {
    pub fn from(art: AssetKey, overview: String, attributes: Vec<(String, String)>) -> InfoPane<T> {
        let mut pane = InfoPane::new();
        pane.container.add_child(Image::from(art), ContainerSize::Percent(0.25));
        let attribute_container = pane.container.with_child(UiContainer::column(), ContainerSize::Fill);
        for (name, value) in &attributes {
            attribute_container.add_child(Label::simple(&format!("{name}: {value}")), ContainerSize::Shrink);
        }
        attribute_container.add_child(Label::wrapping(&overview, None), ContainerSize::Shrink);
        pane
    }
}

impl<T> UiElement<T> for InfoPane<T> {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize { self.container.calc_size(graphics) }

    fn render(&mut self, graphics: &mut Graphics, state: &T, current_focus: &WidgetId) {
        self.container.render(graphics, state, current_focus);
    }

    fn action(
        &mut self,
        _state: &mut T,
        _animations: &mut AnimationManager,
        _action: &Actions,
        _handler: &mut DeferredAction<T>,
    ) -> bool {
        false
    }
}
