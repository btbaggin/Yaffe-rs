use super::{draw_label_and_box, Control, InputControl, LABEL_SIZE};
use crate::input::InputType;
use crate::ui::get_drawable_text;
use crate::utils::Rect;
use crate::{Actions, LogicalPosition};
use winit::keyboard::KeyCode;

pub struct TextBox {
    text: String,
    caret: usize,
    label: String,
    focused: bool,
}
impl TextBox {
    pub fn new(label: String, text: String) -> TextBox { TextBox { label, text, caret: 0, focused: false } }
    pub fn from_str(label: String, text: &str) -> TextBox {
        TextBox { label, text: text.to_string(), caret: 0, focused: false }
    }
}

impl Control for TextBox {
    fn render(&self, graphics: &mut crate::Graphics, container: &Rect) -> crate::LogicalSize {
        const MAX_SIZE: f32 = 500.;

        let size = f32::min(container.width() - LABEL_SIZE - crate::ui::MARGIN, MAX_SIZE);
        let control = draw_label_and_box(graphics, container.top_left(), size, &self.label);

        let height = control.height();
        let text = get_drawable_text(graphics, height, &self.text);
        let box_left = container.left() + LABEL_SIZE;

        let mut cursor_x = 0.;
        let mut origin_x = control.left();
        if self.focused {
            let text = get_drawable_text(graphics, height, &self.text[0..self.caret]);
            // Very special case. The text already accounts for scaling, so we need to undo that to revert back to logical units
            // Then we can do calculations and pass them to the graphics API which converts back to physical units
            let width = text.width() / graphics.scale_factor;

            //If the text is too long to fit in the box, shift it left
            if width > size {
                origin_x = box_left + (size - width)
            }

            cursor_x = f32::min(origin_x + width, control.right());
        }

        //Clip text so it doesnt render outside box
        let clip = Rect::new(
            LogicalPosition::new(box_left, container.top()),
            LogicalPosition::new(box_left + size, container.top() + height),
        );
        graphics.draw_text_cropped(LogicalPosition::new(origin_x, control.top()), clip, graphics.font_color(), &text);

        if self.focused {
            graphics.draw_line(
                LogicalPosition::new(cursor_x, control.top() + 2.),
                LogicalPosition::new(cursor_x, control.bottom() - 2.),
                2.,
                graphics.font_color(),
            );
        }

        crate::LogicalSize::new(control.width() + LABEL_SIZE, control.height())
    }

    fn action(&mut self, action: &Actions) {
        match action {
            Actions::KeyPress(InputType::Key(k, text)) => match k {
                KeyCode::Backspace => {
                    if self.caret > 0 {
                        self.text.remove(self.caret - 1);
                        self.caret -= 1;
                    }
                }
                KeyCode::Delete => {
                    if self.caret < self.text.len() {
                        self.text.remove(self.caret);
                    }
                }
                KeyCode::Home => self.caret = 0,
                KeyCode::End => self.caret = self.text.len(),
                _ => {
                    if let Some(text) = text {
                        self.text.insert_str(self.caret, text);
                        self.caret += text.chars().count();
                    }
                }
            },
            // Actions::KeyPress(InputType::Char(c)) => {
                    // self.text.insert(self.caret, *c);
                    // self.caret += 1;
            // }
            Actions::KeyPress(InputType::Paste(t)) => {
                self.text.insert_str(self.caret, t);
                self.caret += t.len();
            }
            Actions::Right => {
                if self.caret < self.text.len() {
                    self.caret += 1
                }
            }
            Actions::Left => {
                if self.caret > 0 {
                    self.caret -= 1
                }
            }
            _ => {}
        }
    }
}
impl InputControl for TextBox {
    fn value(&self) -> &str { &self.text }
    fn set_focused(&mut self, value: bool) { self.focused = value; }
}
