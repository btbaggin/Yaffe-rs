use crate::{colors::*, Actions, V2, Rectangle, font::FONT_SIZE, ui::*};
use crate::widgets::get_drawable_text;
use crate::settings::SettingsFile;
use speedy2d::Graphics2D;
use crate::modals::outline_rectangle;
use crate::input::InputType;
use crate::windowing::Rect;

pub trait UiControl {
    fn render(&self, graphics: &mut Graphics2D, settings: &SettingsFile, container: &Rectangle, label: &str, focused: bool);
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
}

impl UiControl for TextBox {
    fn render(&self, graphics: &mut Graphics2D, settings: &SettingsFile, container: &Rectangle, label: &str, focused: bool) {
        let size = container.width() - LABEL_SIZE;
        let control = draw_label_and_box(graphics, settings, container.top_left(), size, label, focused);

        let text = get_drawable_text(FONT_SIZE, &self.text);
        graphics.draw_text(*control.top_left(), get_font_color(settings), &text);

        if focused {
            let x = control.left() + self.caret as f32 * 5.;
            graphics.draw_line(V2::new(x, control.top() + 2.), V2::new(x, control.bottom() - 2.), 2., get_font_color(settings));
        }
    }

    fn action(&mut self, action: &Actions) {
        match action {
            Actions::KeyPress(InputType::Key(c)) => self.text.push(*c),
            Actions::KeyPress(InputType::Paste(t)) => self.text.push_str(t),
            Actions::KeyPress(InputType::Delete) => { self.text.pop(); },
            Actions::Right => if self.caret < self.text.len() - 1 { self.caret += 1 },
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

    fn action(&mut self, action: &Actions) {
        if let Actions::Select = action {
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