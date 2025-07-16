use super::{draw_label_and_box, Control, InputControl, LABEL_SIZE};
use crate::input::InputType;
use crate::ui::get_drawable_text;
use crate::utils::Rect;
use crate::{Actions, LogicalPosition};
use copypasta::ClipboardProvider;
use winit::keyboard::{KeyCode, ModifiersState};

pub struct TextBox {
    text: String,
    caret: usize,
    label: String,
    focused: bool,
    selection: Option<(usize, usize)>,
}
impl TextBox {
    pub fn new(label: String, text: String) -> TextBox {
        TextBox { label, text, caret: 0, focused: false, selection: None }
    }
    pub fn from_str(label: String, text: &str) -> TextBox {
        TextBox { label, text: text.to_string(), caret: 0, focused: false, selection: None }
    }
}

impl Control for TextBox {
    fn render(&self, graphics: &mut crate::Graphics, container: &Rect) -> crate::LogicalSize {
        const MAX_SIZE: f32 = 500.;
        const CURSOR_WIDTH: f32 = 2.;

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

        if let Some((start, end)) = self.selection {
            let pre_text = get_drawable_text(graphics, height, &self.text[0..start]);
            let text = get_drawable_text(graphics, height, &self.text[start..end]);
            let pre_width = pre_text.width() / graphics.scale_factor;
            let width = text.width() / graphics.scale_factor;

            let selection_x = f32::max(origin_x + pre_width, control.left());
            let selection_right = f32::min(origin_x + pre_width + width, control.right());

            graphics.draw_rectangle(
                Rect::new(
                    LogicalPosition::new(selection_x, container.top()),
                    LogicalPosition::new(selection_right, container.top() + height),
                ),
                graphics.accent_color(),
            );
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
                CURSOR_WIDTH,
                graphics.font_color(),
            );
        }

        crate::LogicalSize::new(control.width() + LABEL_SIZE, control.height())
    }

    fn action(&mut self, action: &Actions) {
        if let Actions::KeyPress(InputType::Key(k, text, mods)) = action {
            match k {
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
                KeyCode::KeyV if is_command(mods) => {
                    let Ok(mut ctx) = copypasta::ClipboardContext::new() else {
                        return;
                    };
                    let Ok(text) = ctx.get_contents() else {
                        return;
                    };
                    self.insert_text(&text);
                }
                KeyCode::KeyC if is_command(mods) => {
                    let Ok(mut ctx) = copypasta::ClipboardContext::new() else {
                        return;
                    };
                    let _ = ctx.set_contents(self.text.clone());
                }
                KeyCode::ArrowLeft => {
                    let old_caret = self.caret;
                    if self.caret > 0 {
                        if is_command(mods) {
                            self.caret = find_word_boundary(&self.text, self.caret, false)
                        } else {
                            self.caret -= 1
                        }
                    }
                    if is_shift(mods) {
                        self.selection = calculate_selection(self.selection, self.caret, old_caret);
                    } else {
                        self.selection = None;
                    }
                }
                KeyCode::ArrowRight => {
                    let old_caret = self.caret;
                    if self.caret < self.text.len() {
                        if is_command(mods) {
                            self.caret = find_word_boundary(&self.text, self.caret, true)
                        } else {
                            self.caret += 1
                        }
                    }
                    if is_shift(mods) {
                        self.selection = calculate_selection(self.selection, self.caret, old_caret);
                    } else {
                        self.selection = None;
                    }
                }
                _ => {
                    if let Some(text) = text {
                        self.insert_text(text);
                    }
                }
            }
        }
    }
}
impl InputControl for TextBox {
    fn value(&self) -> &str { &self.text }
    fn set_focused(&mut self, value: bool) { self.focused = value; }
}
impl TextBox {
    fn insert_text(&mut self, text: &str) {
        self.text.insert_str(self.caret, text);
        self.caret += text.len();
    }
}

fn is_command(modifiers: &Option<ModifiersState>) -> bool {
    modifiers.is_some_and(|m| {
        if cfg!(target_os = "macos") {
            m.super_key()
        } else if cfg!(not(target_os = "macos")) {
            m.control_key()
        } else {
            false
        }
    })
}

fn is_shift(modifiers: &Option<ModifiersState>) -> bool { modifiers.is_some_and(|m| m.shift_key()) }

fn calculate_selection(
    selection: Option<(usize, usize)>,
    current_caret_position_after_moving: usize,
    position_before_moving: usize,
) -> Option<(usize, usize)> {
    match selection {
        // No existing selection - create new selection from where caret was to where it is now
        None => {
            if current_caret_position_after_moving != position_before_moving {
                // Create selection spanning from old position to new position
                let start = position_before_moving.min(current_caret_position_after_moving);
                let end = position_before_moving.max(current_caret_position_after_moving);
                Some((start, end))
            } else {
                // No movement, no selection
                None
            }
        }

        Some((start, end)) => {
            // Existing selection - extend it based on which end the caret was at
            // We need to determine if the caret was at the start or end of the selection
            let caret_was_at_start = position_before_moving == start;
            let caret_was_at_end = position_before_moving == end;

            if caret_was_at_start {
                // Caret was at start, so extend/shrink from the start
                let new_start = current_caret_position_after_moving.min(end);
                let new_end = current_caret_position_after_moving.max(end);

                if new_start == new_end {
                    None
                } else {
                    Some((new_start, new_end))
                }
            } else if caret_was_at_end {
                // Caret was at end, so extend/shrink from the end
                let new_start = start.min(current_caret_position_after_moving);
                let new_end = start.max(current_caret_position_after_moving);

                if new_start == new_end {
                    None
                } else {
                    Some((new_start, new_end))
                }
            } else {
                // Caret wasn't at either end of selection (shouldn't happen in normal usage)
                // Default to extending from the closest end
                let dist_to_start = position_before_moving.abs_diff(start);
                let dist_to_end = position_before_moving.abs_diff(end);

                if dist_to_start <= dist_to_end {
                    // Closer to start, extend from start
                    let new_start = current_caret_position_after_moving.min(end);
                    let new_end = current_caret_position_after_moving.max(end);
                    Some((new_start, new_end))
                } else {
                    // Closer to end, extend from end
                    let new_start = start.min(current_caret_position_after_moving);
                    let new_end = start.max(current_caret_position_after_moving);
                    Some((new_start, new_end))
                }
            }
        }
    }
}

pub fn find_word_boundary(text: &str, position: usize, direction: bool) -> usize {
    let chars: Vec<char> = text.chars().collect();
    let len = chars.len();
    let mut pos = position;

    if len == 0 {
        return 0;
    }

    if direction {
        if position >= len {
            return text.len();
        }

        // If we're in a word, skip to the end of it, then skip puncuation
        while pos < len && is_word_char(chars[pos]) {
            pos += 1;
        }
        while pos < len && !is_word_char(chars[pos]) {
            pos += 1;
        }
    } else {
        if position == 0 {
            return 0;
        }
        pos = if pos > 0 { pos - 1 } else { 0 };

        // Skip puncuation then if we're in a word, skip to the end of it
        while pos > 0 && !is_word_char(chars[pos]) {
            pos -= 1;
        }
        while pos > 0 && is_word_char(chars[pos]) {
            pos -= 1;
        }
    }
    pos
}

#[inline]
fn is_word_char(c: char) -> bool { c.is_ascii_alphanumeric() || c == '_' }
