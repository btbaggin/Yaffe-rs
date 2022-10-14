use crate::Actions;

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

    /// Provides default handling for actions
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

    /// Returns the number of fields in the focus group
    pub fn len(&self) -> usize {
        self.control.len()
    }

    /// Adds a new field to the focus group
    pub fn insert(&mut self, tag: &str, control: Box<T>) {
        if self.focus == std::ptr::null() {
            self.focus = &control as *const Box<T>;
        }
        self.control.push((tag.to_string(), control));
    }

    /// Retrieves a field from the group based on the tag
    pub fn by_tag(&self, tag: &str) -> Option<&Box<T>> {
        for (t, control) in &self.control {
            if t == tag {
                return Some(control);
            }
        }
        None
    }

    /// Moves the focus to the next field in the group
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

    /// Gets the field that currently has focus
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
    type IntoIter = FocusGroupIter<'a, T>;

    fn into_iter(self) -> Self::IntoIter {
        FocusGroupIter {
            group: self,
            index: 0,
        }
    }
}

pub struct FocusGroupIter<'a, T: ?Sized> {
    group: &'a FocusGroup<T>,
    index: usize,
}

impl<'a, T: ?Sized> Iterator for FocusGroupIter<'a, T> {
    type Item = &'a (String, Box<T>);
    fn next(&mut self) -> Option<Self::Item> {
        let result = self.group.control.get(self.index);
        self.index += 1;
        result
    }
}