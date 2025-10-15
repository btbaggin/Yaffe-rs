use crate::assets::Images;
use crate::ui::{
    change_brightness, AnimationManager, DeferredAction, 
    LayoutElement, RightAlignment, UiContainer, UiElement, WidgetId, MARGIN,
};
use crate::{Actions, Graphics, LogicalPosition, LogicalSize, Rect};
use std::marker::PhantomData;
use std::ops::{Deref, DerefMut};

use super::{ModalClose, ModalInputHandler};

crate::widget!(
    pub struct ModalTitlebar<T> {
        title: String = String::new(),
        _data: PhantomData<T> = PhantomData
    }
);
impl<T> ModalTitlebar<T> {
    pub fn from(title: String) -> ModalTitlebar<T> {
        let mut titlebar = ModalTitlebar::new();
        titlebar.title = title;
        titlebar
    }
}
impl<T> UiElement<T> for ModalTitlebar<T> {
    fn render(&mut self, graphics: &mut Graphics, _: &T, _: &WidgetId) {
        let layout = self.layout();
        const PADDING: f32 = 2.;
        let titlebar_color = graphics.accent_color();
        let titlebar_color = change_brightness(&titlebar_color, graphics.light_shade_factor());

        let pos = *layout.top_left();
        let pos = LogicalPosition::new(pos.x + PADDING, pos.y + PADDING);
        let titlebar = Rect::point_and_size(pos, layout.size() - LogicalSize::new(PADDING * 2., PADDING));
        graphics.draw_rectangle(titlebar, titlebar_color);

        let title_text = crate::ui::get_drawable_text(graphics, layout.height(), &self.title);
        let title_pos = *layout.top_left() + LogicalPosition::new(MARGIN + PADDING, PADDING);
        graphics.draw_text(title_pos, graphics.font_color(), &title_text);
    }
}

#[allow(unused_variables)]
pub struct ModalContentElement<T: 'static> {
    position: LogicalPosition,
    size: LogicalSize,
    id: WidgetId,
    focus_group: bool,
    focus: Option<WidgetId>,
    handler: Box<dyn ModalInputHandler<T>>,
    container: UiContainer<T>,
}
impl<T: 'static> LayoutElement for ModalContentElement<T> {
    fn layout(&self) -> Rect { Rect::point_and_size(self.position, self.size) }
    fn set_layout(&mut self, layout: Rect) {
        self.position = *layout.top_left();
        self.size = layout.size();
    }
    fn get_id(&self) -> WidgetId { self.id }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
impl<T: 'static> ModalContentElement<T> {
    pub fn new(handler: impl ModalInputHandler<T> + 'static, focus_group: bool) -> ModalContentElement<T> {
        ModalContentElement {
            position: LogicalPosition::new(0., 0.),
            size: LogicalSize::new(0., 0.),
            id: WidgetId::random(),
            focus_group,
            focus: None,
            handler: Box::new(handler),
            container: UiContainer::column(),
        }
    }
    pub fn get_handler<H: 'static>(&self) -> &H { crate::convert_to!(&self.handler, H) }

    pub fn focus(&mut self, id: WidgetId) { self.focus = Some(id); }
}
impl<T: 'static> Deref for ModalContentElement<T> {
    type Target = UiContainer<T>;
    fn deref(&self) -> &Self::Target { &self.container }
}
impl<T: 'static> DerefMut for ModalContentElement<T> {
    fn deref_mut(&mut self) -> &mut Self::Target { &mut self.container }
}

impl<T: 'static> UiElement<T> for ModalContentElement<T> {
    fn as_container(&self) -> Option<&UiContainer<T>> { Some(&self.container) }
    fn as_container_mut(&mut self) -> Option<&mut UiContainer<T>> { Some(&mut self.container) }

    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize { self.container.calc_size(graphics) }

    fn render(&mut self, graphics: &mut Graphics, state: &T, current_focus: &WidgetId) {
        let rect = self.layout();
        graphics.bounds = Rect::point_and_size(
            LogicalPosition::new(rect.left() + MARGIN, rect.top()),
            LogicalSize::new(rect.width() - (MARGIN * 2.), rect.height()),
        );
        self.container.render(graphics, state, &self.focus.unwrap_or(*current_focus));
    }

    fn action(
        &mut self,
        state: &mut T,
        animations: &mut AnimationManager,
        action: &Actions,
        handler: &mut DeferredAction<T>,
    ) -> bool {
        // See if we should close
        if ModalClose::close_if_accept(action, handler) {
            return true;
        }
        // If we have a focus group, move focus first
        if self.focus_group {
            match action {
                Actions::Up => {
                    self.focus = self.container.move_focus(self.focus, false);
                    return true;
                }
                Actions::Down => {
                    self.focus = self.container.move_focus(self.focus, true);
                    return true;
                }
                _ => {}
            }
        }
        // If current control is focused, handle that
        if let Some(focus) = self.focus {
            if let Some(widget) = self.container.find_widget_mut(focus) {
                return widget.action(state, animations, action, handler);
            }
        }

        // Otherwise custom handling
        if self.handler.action(state, animations, action, handler, &mut self.container) {
            return true;
        }
        self.container.action(state, animations, action, handler)
    }
}

crate::widget!(
    pub struct ModalToolbar<T> {
        confirmation_button: String = String::new(),
        _data: PhantomData<T> = PhantomData
    }
);
impl<T> ModalToolbar<T> {
    pub fn from(confirm: String) -> ModalToolbar<T> {
        let mut content = ModalToolbar::<T>::new();
        content.confirmation_button = confirm;
        content
    }
}
impl<T> crate::ui::UiElement<T> for ModalToolbar<T> {
    fn render(&mut self, graphics: &mut Graphics, _: &T, _: &WidgetId) {
        let rect = self.layout();

        let right = LogicalPosition::new(rect.right() - MARGIN, rect.top());
        let image_size = LogicalSize::new(graphics.font_size(), graphics.font_size());
        let mut alignment = RightAlignment::new(right);
        for t in [("Cancel", Images::ButtonB), (&self.confirmation_button[..], Images::ButtonA)] {
            alignment = alignment.text(graphics, t.0).image(graphics, t.1, image_size).space();
        }
    }
}