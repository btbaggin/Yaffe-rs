use crate::ui::{get_accent_color, get_font_size, UiControl, FocusGroup};
use crate::graphics::Graphics;
use speedy2d::color::Color;
use crate::input::Actions;
use crate::Rect;
use crate::settings::SettingsFile;

enum ContainerDirection {
    Horizontal,
    Vertical,
}

pub struct Container {
    direction: ContainerDirection,
    background: Option<Color>,
    controls: FocusGroup<dyn UiControl>,

}
impl Container {
    pub fn horizontal(background: Option<Color>) -> Container {
        Container {
            background, direction: ContainerDirection::Horizontal, controls: FocusGroup::new()
        }
    }
    pub fn vertical(background: Option<Color>) -> Container {
        Container {
            background, direction: ContainerDirection::Vertical, controls: FocusGroup::new()
        }
    }
}
impl UiControl for Container {
    //TODO for this to work I would need to know the size of inner elements after rendering. Not terribly difficult but a change
    fn render(&self, graphics: &mut crate::Graphics, settings: &SettingsFile, container: &Rect, label: &str, focused: bool) {
        // let control = draw_label_and_box(graphics, settings, &container.top_left(), get_font_size(settings, graphics), label, focused);

        // if self.checked {
        //     let base = get_accent_color(settings);
        //     graphics.draw_rectangle(Rect::from_tuples((control.left() + 4., control.top() + 4.), (control.right() - 4., control.bottom() - 4.)), base)
        // }
    }

    //TODO would need a new abstraction because this wont be true for containers
    fn value(&self) -> &str {
        ""
    }

    fn action(&mut self, action: &Actions) {
        if !self.controls.action(action) {
            if let Some(focus) = self.controls.focus() {
                focus.action(action);
            }
        }
    }
}

// impl super::UiElement for DockContainer {
//     fn as_any(&self) -> &dyn std::any::Any { self }
//     fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
    
//     fn render(&self, graphics: &mut Graphics, bounds: &Rectangle) {
//         if let Some(b) = self.background {
//             graphics.draw_rectangle(bounds.clone(), b);
//         }
//     }
//     fn update(&mut self, _state: &mut UpdateState, helper: &mut WidgetHelper, _rect: &Rectangle) { }
//     fn layout(&mut self, rect: &Rectangle, helper: &mut WidgetHelper) -> Rectangle {
//         let width = rect.width() * self.width;
//         let height = rect.height() * self.height;

//         let mut offset = 0.;
//         let child_count = helper.children.len();
        
//         let size = V2::new(width, height);
//         let pos = helper.align(rect, &size);
//         let rect = Rectangle::new(pos, size);
//         for c in helper.children.iter_mut() {
//             offset += match self.direction {
//                 DockDirection::Horizontal => {
//                     let child_rect = Rectangle::new(V2::new(rect.left() + offset, rect.top()), V2::new(width / child_count as f32, height));
//                     c.layout(&child_rect);
//                     c.bounds.width()
//                 },
//                 DockDirection::Vertical => {
//                     let child_rect = Rectangle::new(V2::new(rect.left(), rect.top() + offset), V2::new(width, height / child_count as f32));
//                     c.layout(&child_rect);
//                     c.bounds.height()
//                 }
//             }

//         }

//         rect
//     }
// }