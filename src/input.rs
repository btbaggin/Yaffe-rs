use std::hash::Hash;
use glutin::event::VirtualKeyCode;
use std::collections::HashMap;

pub struct InputMap<A: Eq + Hash, B: Eq + Hash, T: Clone> {
    keys: HashMap<A, T>,
    cont: HashMap<B, T>,
}
impl<A: Eq + Hash, B: Eq + Hash, T: Clone> InputMap<A, B, T> {
    fn new() -> InputMap<A, B, T> {
        InputMap {
            keys: HashMap::new(),
            cont: HashMap::new(),
        }
    }

    fn insert(&mut self, code: A, button: B, action: T) {
        self.keys.insert(code, action.clone());
        self.cont.insert(button, action);
    }

    pub fn get(&self, code: Option<A>, button: Option<B>) -> Option<&T> {
        if let Some(c) = code {
            return self.keys.get(&c);
        } else if let Some(b) = button {
            return self.cont.get(&b);
        }
        return None;
    }
}

#[derive(Clone, Copy, Hash, Eq, PartialEq)]
pub enum Actions {
    Info,
    Accept,
    Select,
    Back,
    Up,
    Down,
    Left,
    Right,
    Filter,
    ToggleOverlay,
    ShowMenu,
    KeyPress(InputType),
}

#[derive(Hash, Eq, PartialEq, Copy, Clone)]
pub enum ControllerInput {
    ButtonNorth,
    ButtonSouth,
    ButtonEast,
    ButtonWest,
    ButtonStart,
    ButtonBack,
    ButtonGuide,
    DirectionLeft,
    DirectionRight,
    DirectionUp,
    DirectionDown,
}

#[derive(Copy, Clone, Eq, PartialEq, Hash)]
pub enum InputType {
    Key(char),
    Delete,
    Paste,
}

pub fn get_input_map() -> InputMap<VirtualKeyCode, ControllerInput, Actions> {
    let mut m = InputMap::new();
    m.insert(VirtualKeyCode::Key1, ControllerInput::ButtonWest, Actions::Info);
    m.insert(VirtualKeyCode::Key2, ControllerInput::ButtonNorth, Actions::Filter);
    m.insert(VirtualKeyCode::Return, ControllerInput::ButtonSouth, Actions::Accept);
    m.insert(VirtualKeyCode::Escape, ControllerInput::ButtonEast, Actions::Back);
    m.insert(VirtualKeyCode::Up, ControllerInput::DirectionUp, Actions::Up);
    m.insert(VirtualKeyCode::Down, ControllerInput::DirectionDown, Actions::Down);
    m.insert(VirtualKeyCode::Right, ControllerInput::DirectionRight, Actions::Right);
    m.insert(VirtualKeyCode::Left, ControllerInput::DirectionLeft, Actions::Left);
    m.insert(VirtualKeyCode::Tab, ControllerInput::ButtonBack, Actions::Select);
    m.insert(VirtualKeyCode::F1, ControllerInput::ButtonStart, Actions::ShowMenu);
    m.insert(VirtualKeyCode::F2, ControllerInput::ButtonGuide, Actions::ToggleOverlay);
    m
}

pub trait PlatformInput {
    fn update(&mut self, controller_index: u32) -> Result<(), u32>;
    fn get_keyboard(&mut self) -> Vec<(VirtualKeyCode, Option<char>)>;
    fn get_gamepad(&mut self) -> Vec<ControllerInput>;
    fn get_modifiers(&mut self) -> glutin::event::ModifiersState;
}