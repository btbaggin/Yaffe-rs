use super::{Control, InputControl, draw_label_and_box, get_font_color, LABEL_SIZE};
use crate::{Actions, LogicalPosition};
use crate::ui::get_drawable_text;
use crate::settings::SettingsFile;
use crate::input::InputType;
use crate::utils::Rect;
use glutin::event::VirtualKeyCode;

pub struct TextBox {
    text: String,
    caret: usize,
    label: String,
    focused: bool,
}
impl TextBox {
    pub fn new(label: String, text: String) -> TextBox {
        TextBox { label, text, caret: 0, focused: false }
    }
    pub fn from_str(label: String, text: &str) -> TextBox {
        TextBox { label, text: text.to_string(), caret: 0, focused: false }
    }
}

impl Control for TextBox {
    fn render(&self, graphics: &mut crate::Graphics, settings: &SettingsFile, container: &Rect) -> crate::LogicalSize {
        const MAX_SIZE: f32 = 250.;

        let size = f32::min(container.width() - LABEL_SIZE, MAX_SIZE);
        let control = draw_label_and_box(graphics, settings, container.top_left(), size, &self.label);

        let height = control.height();
        let text = get_drawable_text(height, &self.text);
        let box_left = container.left() + LABEL_SIZE;

        let mut cursor_x = 0.;
        let mut origin_x = control.left();
        if self.focused {
            let text = get_drawable_text(height, &self.text[0..self.caret]);

            //If the text is too long to fit in the box, shift it left
            if text.width() > size {
                origin_x = box_left + (size - text.width())
            } 

            cursor_x = f32::min(origin_x + text.width(), control.right());
        }

        //Clip text so it doesnt render outside box
        let clip = Rect::new(LogicalPosition::new(box_left, container.top()), LogicalPosition::new(container.right(), container.top() + height));
        graphics.set_clip(Some(clip));
        graphics.draw_text(LogicalPosition::new(origin_x, control.top()), get_font_color(settings), &text);
        graphics.set_clip(None);

        if self.focused {
            graphics.draw_line(LogicalPosition::new(cursor_x, control.top() + 2.), LogicalPosition::new(cursor_x, control.bottom() - 2.), 2., get_font_color(settings));
        }

        crate::LogicalSize::new(control.width() + LABEL_SIZE, control.height())
    }

    fn action(&mut self, action: &Actions) {
        match action {
            Actions::KeyPress(InputType::Key(k)) => {
                match k {
                    VirtualKeyCode::Back => {
                        if self.caret > 0 {
                            self.text.remove(self.caret - 1);
                            self.caret -= 1;
                        }
                    },
                    VirtualKeyCode::Delete => {
                        if self.caret < self.text.len() {
                            self.text.remove(self.caret);
                        }
                    },
                    VirtualKeyCode::Home => self.caret = 0,
                    VirtualKeyCode::End => self.caret = self.text.len(),
                    _ => {},
                }
            }
            Actions::KeyPress(InputType::Char(c)) => {
                self.text.insert(self.caret, *c);
                self.caret += 1;
            }
            Actions::KeyPress(InputType::Paste(t)) => {
                self.text.insert_str(self.caret, t);
                self.caret += t.len();
            }
            Actions::Right => if self.caret < self.text.len() { self.caret += 1 },
            Actions::Left => if self.caret > 0 { self.caret -= 1 },
            _ => {},
        }
    }
}
impl InputControl for TextBox {
    fn value(&self) -> &str { &self.text }
    fn set_focused(&mut self, value: bool) { self.focused = value; }
}