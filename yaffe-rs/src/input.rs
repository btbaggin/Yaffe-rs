use std::collections::HashMap;
use std::hash::Hash;
use winit::keyboard::KeyCode;

pub struct InputMap<A: Eq + Hash, B: Eq + Hash, T: Clone> {
    keys: HashMap<A, T>,
    cont: HashMap<B, T>,
}
impl<A: Eq + Hash, B: Eq + Hash, T: Clone> InputMap<A, B, T> {
    fn new() -> InputMap<A, B, T> { InputMap { keys: HashMap::new(), cont: HashMap::new() } }

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
        None
    }
}

#[derive(Clone, Hash, Eq, PartialEq)]
pub enum Actions {
    Info,
    Accept,
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

#[derive(Clone, Eq, PartialEq, Hash)]
pub enum InputType {
    Key(KeyCode, Option<String>),
    Paste(String),
}

pub fn get_input_map() -> InputMap<KeyCode, ControllerInput, Actions> {
    let mut m = InputMap::new();
    m.insert(KeyCode::Digit1, ControllerInput::ButtonWest, Actions::Info);
    m.insert(KeyCode::Digit2, ControllerInput::ButtonNorth, Actions::Filter);
    m.insert(KeyCode::Enter, ControllerInput::ButtonSouth, Actions::Accept);
    m.insert(KeyCode::Escape, ControllerInput::ButtonEast, Actions::Back);
    m.insert(KeyCode::ArrowUp, ControllerInput::DirectionUp, Actions::Up);
    m.insert(KeyCode::ArrowDown, ControllerInput::DirectionDown, Actions::Down);
    m.insert(KeyCode::ArrowRight, ControllerInput::DirectionRight, Actions::Right);
    m.insert(KeyCode::ArrowLeft, ControllerInput::DirectionLeft, Actions::Left);
    m.insert(KeyCode::F1, ControllerInput::ButtonStart, Actions::ShowMenu);
    m.insert(KeyCode::F2, ControllerInput::ButtonGuide, Actions::ToggleOverlay);
    m
}

pub trait PlatformGamepad {
    fn update(&mut self, controller_index: u32) -> Result<(), u32>;
    fn get_gamepad(&mut self) -> Vec<ControllerInput>;
}

pub fn input_to_action(
    input_map: &InputMap<KeyCode, ControllerInput, Actions>,
    input: &mut dyn PlatformGamepad,
) -> std::collections::HashSet<Actions> {
    let mut result = std::collections::HashSet::new();
    for g in input.get_gamepad() {
        if let Some(action) = input_map.get(None, Some(g)) {
            result.insert(action.clone());
        } else {
            result.insert(Actions::KeyPress(InputType::Key(KeyCode::Backquote, Some((g as u8 as char).to_string()))));
        }
    }

    result
}
