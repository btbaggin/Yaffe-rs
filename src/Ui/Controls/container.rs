use crate::ui::{Control, InputControl, MARGIN};
use crate::input::Actions;
use crate::{Rect, LogicalSize};
use crate::settings::SettingsFile;
use std::collections::HashMap;

enum ContainerDirection {
    Horizontal,
    Vertical,
}

enum ContainerType {
    Input(Box<dyn InputControl>),
    Control(Box<dyn Control>)
}

pub struct Container {
    direction: ContainerDirection,
    controls: Vec<ContainerType>,
    tags: HashMap<String, usize>,
    focus_index: Option<usize>,
    size: f32
}
impl Container {
    pub fn horizontal(height: f32) -> Container {
        Container {
            direction: ContainerDirection::Horizontal,
            controls: vec!(),
            tags: HashMap::new(),
            focus_index: None,
            size: height,
        }
    }
    pub fn vertical(width: f32) -> Container {
        Container {
            direction: ContainerDirection::Vertical,
            controls: vec!(),
            tags: HashMap::new(),
            focus_index: None,
            size: width,
        }
    }
}
impl Container {
    pub fn child_count(&self) -> usize {
        self.controls.len()
    }

    /// Adds a new field to the focus group
    pub fn add_field(&mut self, tag: &str, control: impl InputControl + 'static) {
        let i = self.controls.len();
        self.controls.push(ContainerType::Input(Box::new(control)));
        self.tags.insert(tag.to_string(), i);
    }

    pub fn add(&mut self, control: impl Control + 'static) {
        self.controls.push(ContainerType::Control(Box::new(control)));
    }

    /// Retrieves a field from the group based on the tag
    pub fn by_tag(&self, tag: &str) -> Option<&Box<dyn InputControl>> {
        if let Some(i) = self.tags.get(tag) {
            match &self.controls[*i] {
                ContainerType::Input(i) => return Some(i),
                ContainerType::Control(_) => panic!("This shouldn't happen"),
            }
        } 
        //TODO check child containers
        None
    }

    /// Moves the focus to the next field in the group
    fn move_focus(&mut self, next: bool) {
        //Try to find current focus
        //Move index based on index and if it exists
        let mut index = match self.focus_index {
            None => if next { 0 } else { self.child_count() - 1 },
            Some(index) => {
                self.set_focus(index, false);
                if next { index + 1 } 
                else { 
                    if index == 0 { self.child_count() - 1}
                    else { index - 1 }
                }
            }
        };

        while let Some(ContainerType::Control(_)) = self.controls.get(index) {
            index = if next { index + 1 } else { index - 1 };
        }

        if index == self.controls.len() {
            self.focus_index = None;
        } else {
            self.focus_index = Some(index);
            self.set_focus(index, true);
        }
    }

    fn set_focus(&mut self, index: usize, value: bool) {
        match &mut self.controls[index] {
            ContainerType::Input(i) => i.set_focused(value),
            ContainerType::Control(_) => panic!("This should not have happened"),
        }
    }
}
impl Control for Container {
    fn render(&self, graphics: &mut crate::Graphics, settings: &SettingsFile, container: &Rect) -> LogicalSize {
        //TODO this causes double margins. Can i fix that? thats like... nice? Because there are hacky approaces
        let top_left = *container.top_left() + crate::LogicalPosition::new(MARGIN, MARGIN);
        match self.direction {
            ContainerDirection::Vertical => {
                let container_size = container.width() * self.size;
                let mut rect = Rect::point_and_size(top_left, LogicalSize::new(container_size, container.height()));
                let mut y = container.top();

                for (i, v) in self.controls.iter().enumerate() {
                    let size = render_control(graphics, settings, rect, self.focus_index, v, i);
        
                    y += size.y + MARGIN;
                    rect = Rect::from_tuples((top_left.x, y), (rect.right(), rect.bottom()));
                    
                }

                LogicalSize::new(container_size, y - container.top())
            },
            ContainerDirection::Horizontal => {
                let container_size = container.height() * self.size;
                let mut rect = Rect::point_and_size(top_left, LogicalSize::new(container.width(), container_size));
                let mut x = container.left();

                for (i, v) in self.controls.iter().enumerate() {
                    let size = render_control(graphics, settings, rect, self.focus_index, v, i);
        
                    x += size.x + MARGIN;
                    rect = Rect::from_tuples((x, top_left.y), (rect.right(), rect.bottom()));
                }

                LogicalSize::new(x - container.left(), container_size)
            },
        }
    }

    fn action(&mut self, action: &Actions) {
        let handled = match action {
            Actions::Up => {
                self.move_focus(false);
                true
            },
            Actions::Down => {
                self.move_focus(true);
                true
            },
            _ => false,
        };

        if !handled {
            if let Some(i) = self.focus_index {
                match &mut self.controls[i] {
                    ContainerType::Input(i) => i.action(action),
                    ContainerType::Control(c) => c.action(action),
                }
            }
        }
    }
}

fn render_control(graphics: &mut crate::Graphics,
                  settings: &SettingsFile,
                  rect: Rect,
                  focus_index: Option<usize>,
                  control: &ContainerType,
                  current_index: usize) -> crate::LogicalSize {
    let size = match control {
        ContainerType::Input(i) => i.render(graphics, settings, &rect),
        ContainerType::Control(c) => c.render(graphics, settings, &rect),
    };

    if let Some(index) = focus_index {
        if index == current_index {
            let min = *rect.top_left();
            let max = *rect.top_left() + size;
        
            let control = Rect::new(min, max);
            let base = crate::ui::get_accent_color(settings);
            let light_factor = settings.get_f32(crate::SettingNames::LightShadeFactor);
            crate::ui::outline_rectangle(graphics, &control, 1., crate::ui::change_brightness(&base, light_factor));
        }
    }

    size
}