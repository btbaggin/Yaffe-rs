use super::{UiControl, draw_label_and_box, get_font_color, get_font_size, LABEL_SIZE};
use crate::{Actions, LogicalPosition};
use crate::ui::get_drawable_text;
use crate::settings::SettingsFile;
use crate::input::InputType;
use crate::utils::Rect;
use glutin::event::VirtualKeyCode;

pub struct TextBox {
    text: String,
    caret: usize,
}
impl TextBox {
    pub fn new(text: String) -> TextBox {
        TextBox { text, caret: 0 }
    }
    pub fn from_str(text: &str) -> TextBox {
        TextBox { text: text.to_string(), caret: 0 }
    }
}

impl UiControl for TextBox {
    fn render(&self, graphics: &mut crate::Graphics, settings: &SettingsFile, container: &Rect, label: &str, focused: bool) {
        let font_size = get_font_size(settings, graphics);
        let size = container.width() - LABEL_SIZE;
        let control = draw_label_and_box(graphics, settings, &container.top_left(), size, label, focused);

        let text = get_drawable_text(font_size, &self.text);
        let box_left = container.left() + LABEL_SIZE;

        let mut cursor_x = 0.;
        let mut origin_x = control.left();
        if focused {
            let text = get_drawable_text(font_size, &self.text[0..self.caret]);

            //If the text is too long to fit in the box, shift it left
            if text.width() > size {
                origin_x = box_left + (size - text.width())
            } 

            cursor_x = f32::min(origin_x + text.width(), control.right());
        }

        //Clip text so it doesnt render outside box
        let clip = Rect::new(LogicalPosition::new(box_left, container.top()), LogicalPosition::new(container.right(), container.bottom()));
        graphics.set_clip(Some(clip));
        graphics.draw_text(LogicalPosition::new(origin_x, control.top()), get_font_color(settings), &text);
        graphics.set_clip(None);

        if focused {
            graphics.draw_line(LogicalPosition::new(cursor_x, control.top() + 2.), LogicalPosition::new(cursor_x, control.bottom() - 2.), 2., get_font_color(settings));
        }
    }

    fn value(&self) -> &str {
        &self.text
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