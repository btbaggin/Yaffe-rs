use crate::{colors::*, Actions, V2, Rectangle, font::FONT_SIZE, ui::*};
use crate::widgets::get_drawable_text;
use crate::settings::SettingsFile;
use speedy2d::Graphics2D;
use crate::modals::outline_rectangle;
use crate::input::InputType;
use crate::windowing::Rect;
use glutin::event::VirtualKeyCode;

pub struct FocusGroup<T: ?Sized> {
    control: Vec<(String, Box<T>)>,
    focus: *const Box<T>,
}
impl<T: ?Sized> FocusGroup<T> {
    pub fn new() -> FocusGroup<T> {
        FocusGroup { 
            control: vec!(),
            focus: std::ptr::null(),
        }
    }

    pub fn action(&mut self, action: &Actions) -> bool {
        match action {
            Actions::Up => {
                self.move_focus(false);
                true
            },
            Actions::Down => {
                self.move_focus(true);
                true
            },
            _ => false,
        }
    }

    pub fn len(&self) -> usize {
        self.control.len()
    }

    pub fn insert(&mut self, tag: &str, control: Box<T>) {
        if self.focus == std::ptr::null() {
            self.focus = &control as *const Box<T>;
        }
        self.control.push((tag.to_string(), control));
    }

    pub fn by_tag(&self, tag: &str) -> Option<&Box<T>> {
        for (t, control) in &self.control {
            if t == tag {
                return Some(control);
            }
        }
        None
    }

    pub fn move_focus(&mut self, next: bool) {
        //Try to find current focus
        let index = self.control.iter().position(|value| std::ptr::eq(&value.1 as *const Box<T>, self.focus));
        
        //Move index based on index and if it exists
        let index = match index {
            None => if next { 0 } else { self.control.len() - 1 },
            Some(index) => if next { index + 1 } 
            else { 
                if index == 0 { self.control.len() - 1}
                else { index - 1 }
            }
        };

        //Set new focus
        self.focus = match self.control.get(index) { 
            None => std::ptr::null(),
            Some(value) => &value.1 as *const Box<T>,
        }
    }

    pub fn focus(&mut self) -> Option<&mut Box<T>> {
        for c in self.control.iter_mut() {
            let ptr = &c.1 as *const Box<T>;
            if std::ptr::eq(self.focus, ptr) {
                return Some(&mut c.1)
            }
        }
        None
    }

    pub fn is_focused(&self, other: &Box<T>) -> bool {
        std::ptr::eq(self.focus, other as *const Box<T>)
    }
}
impl<'a, T: ?Sized> IntoIterator for &'a FocusGroup<T> {
    type Item = &'a (String, Box<T>);
    type IntoIter = FocusGroupIterator<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        FocusGroupIterator {
            group: self,
            index: 0,
        }
    }
}

pub struct FocusGroupIterator<'a, T: ?Sized> {
    group: &'a FocusGroup<T>,
    index: usize,
}

impl<'a, T: ?Sized> Iterator for FocusGroupIterator<'a, T> {
    type Item = &'a (String, Box<T>);
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.group.control.get(self.index);
        self.index += 1;
        result
    }
}

pub trait UiControl {
    fn render(&self, graphics: &mut Graphics2D, settings: &SettingsFile, container: &Rectangle, label: &str, focused: bool);
    fn value(&self) -> &str;
    fn action(&mut self, action: &Actions);
}

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
    fn render(&self, graphics: &mut Graphics2D, settings: &SettingsFile, container: &Rectangle, label: &str, focused: bool) {
        let size = container.width() - LABEL_SIZE;
        let control = draw_label_and_box(graphics, settings, container.top_left(), size, label, focused);

        let text = get_drawable_text(FONT_SIZE, &self.text);
        graphics.draw_text(*control.top_left(), get_font_color(settings), &text);

        if focused {
            let text = get_drawable_text(FONT_SIZE, &self.text[0..self.caret]);
            let x = control.left() + text.width();
            
            graphics.draw_line(V2::new(x, control.top() + 2.), V2::new(x, control.bottom() - 2.), 2., get_font_color(settings));
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

pub struct CheckBox {
    checked: bool,
}
impl CheckBox {
    pub fn new(checked: bool) -> CheckBox {
        CheckBox { checked }
    }
}

impl UiControl for CheckBox {
    fn render(&self, graphics: &mut Graphics2D, settings: &SettingsFile, container: &Rectangle, label: &str, focused: bool) {
        let control = draw_label_and_box(graphics, settings, container.top_left(), FONT_SIZE, label, focused);

        if self.checked {
            let base = crate::colors::get_accent_color(settings);

            graphics.draw_rectangle(Rectangle::from_tuples((control.left() + 4., control.top() + 4.), (control.right() - 4., control.bottom() - 4.)), base)
        }
    }

    fn value(&self) -> &str {
        if self.checked { "true" } else { "false" }
    }

    fn action(&mut self, action: &Actions) {
        if let Actions::KeyPress(InputType::Key(VirtualKeyCode::Space)) = action {
            self.checked = !self.checked;
        }
    }
}

fn draw_label_and_box(graphics: &mut Graphics2D, settings: &SettingsFile, pos: &V2, size: f32, label: &str, focused: bool) -> Rectangle {
    let label = get_drawable_text(FONT_SIZE, label);
    graphics.draw_text(*pos, get_font_color(settings), &label); 

    let min = V2::new(pos.x + LABEL_SIZE, pos.y);
    let max = V2::new(pos.x + LABEL_SIZE + size, pos.y + FONT_SIZE);

    let control = Rectangle::new(min, max);
    let base = crate::colors::get_accent_color(settings);
    let factor = settings.get_f32(crate::SettingNames::DarkShadeFactor);
    graphics.draw_rectangle(control.clone(), change_brightness(&base, factor));
    
    if focused {
        let light_factor = settings.get_f32(crate::SettingNames::LightShadeFactor);
        outline_rectangle(graphics, &control, 1., change_brightness(&base, light_factor));
    }
    control
}