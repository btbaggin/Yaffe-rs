use crate::input::InputType;
use crate::ui::{AnimationManager, LayoutElement, UiElement, ValueElement, WidgetId};
use crate::{Actions, Graphics, LogicalSize};
use std::hash::{Hash, Hasher};

const PIN_SIZE: usize = 8;

#[derive(Default, Copy, Clone)]
pub struct RestrictedPasscode {
    code: [char; PIN_SIZE],
    length: usize,
}
impl RestrictedPasscode {
    pub fn add_digit(&mut self, code: char) {
        if self.length < PIN_SIZE {
            self.code[self.length] = code;
            self.length += 1;
        }
    }
    pub fn len(&self) -> usize { self.length }
}
impl PartialEq for RestrictedPasscode {
    fn eq(&self, other: &Self) -> bool {
        if self.length != other.length {
            return false;
        }

        for i in 0..self.length {
            if self.code[i] != other.code[i] {
                return false;
            }
        }

        true
    }
}

impl Eq for RestrictedPasscode {}

crate::widget!(
    pub struct PassBox {
        passcode: RestrictedPasscode = RestrictedPasscode::default()
    }
);

impl<T: 'static, D: 'static> UiElement<T, D> for PassBox {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize {
        LogicalSize::new(graphics.bounds.width(), graphics.font_size())
    }
    fn render(&mut self, graphics: &mut Graphics, _: &T, _: &WidgetId) {
        let font_size = graphics.font_size();
        let rect = self.layout();

        let text = String::from_utf8(vec![b'*'; self.passcode.len()]).unwrap();
        let item_label = crate::ui::get_drawable_text(graphics, font_size, &text);
        graphics.draw_text(*rect.top_left(), graphics.font_color(), &item_label);
    }

    fn action(&mut self, _state: &mut T, _: &mut AnimationManager, action: &Actions, _handler: &mut D) -> bool {
        let code = match action {
            Actions::KeyPress(InputType::Key(code, _, _)) => *code as u8 as char,
            Actions::KeyPress(InputType::Gamepad(g)) => *g as u8 as char,
            _ => action_to_char(action),
        };
        self.passcode.add_digit(code);
        false
    }
}
impl ValueElement<RestrictedPasscode> for PassBox {
    fn value(&self) -> RestrictedPasscode { self.passcode }
}

fn action_to_char(action: &Actions) -> char {
    let mut hasher = std::collections::hash_map::DefaultHasher::new();
    action.hash(&mut hasher);
    hasher.finish() as u8 as char
}
