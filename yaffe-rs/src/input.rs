use std::hash::Hash;
use std::{collections::HashMap, time::Instant};
use winit::keyboard::{KeyCode, ModifiersState};

use crate::logger::PanicLogEntry;

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
    Key(KeyCode, Option<String>, Option<ModifiersState>),
    Gamepad(ControllerInput),
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
    fn update(&mut self) -> Result<(), Box<dyn std::error::Error>>;
    fn is_button_pressed(&self, button: ControllerInput) -> bool;
    fn get_left_thumbstick(&self) -> (f32, f32);
}

pub struct Gamepad {
    platform: Box<dyn PlatformGamepad + 'static>,
    last_input: Instant,
}
impl Gamepad {
    pub fn new() -> Gamepad {
        let platform_impl = crate::os::initialize_gamepad().log_message_and_panic("Unable to initialize input");
        Gamepad { platform: Box::new(platform_impl), last_input: Instant::now() }
    }
    pub fn update(&mut self) -> Result<(), Box<dyn std::error::Error>> { self.platform.update() }
}

pub fn input_to_action(
    input_map: &InputMap<KeyCode, ControllerInput, Actions>,
    input: &mut Gamepad,
) -> std::collections::HashSet<Actions> {
    let mut result = std::collections::HashSet::new();
    add_thumbstick_actions(input, input_map, &mut result);

    if input.platform.is_button_pressed(ControllerInput::ButtonStart) {
        add_action(ControllerInput::ButtonStart, input_map, &mut result);
    }
    if input.platform.is_button_pressed(ControllerInput::ButtonBack) {
        add_action(ControllerInput::ButtonBack, input_map, &mut result);
    }
    if input.platform.is_button_pressed(ControllerInput::ButtonSouth) {
        add_action(ControllerInput::ButtonSouth, input_map, &mut result);
    }
    if input.platform.is_button_pressed(ControllerInput::ButtonEast) {
        add_action(ControllerInput::ButtonEast, input_map, &mut result);
    }
    if input.platform.is_button_pressed(ControllerInput::ButtonWest) {
        add_action(ControllerInput::ButtonWest, input_map, &mut result);
    }
    if input.platform.is_button_pressed(ControllerInput::ButtonNorth) {
        add_action(ControllerInput::ButtonNorth, input_map, &mut result);
    }

    result
}

fn add_thumbstick_actions(
    gamepad: &mut Gamepad,
    input_map: &InputMap<KeyCode, ControllerInput, Actions>,
    result: &mut std::collections::HashSet<Actions>,
) {
    let now = Instant::now();
    if (now - gamepad.last_input).as_millis() > 100 {
        let (x, y) = gamepad.platform.get_left_thumbstick();
        if x < 0.0 && x.abs() > y.abs() {
            add_action(ControllerInput::DirectionLeft, input_map, result);
            gamepad.last_input = now;
        }
        if y > 0.0 && y.abs() > x.abs() {
            add_action(ControllerInput::DirectionUp, input_map, result);
            gamepad.last_input = now;
        }
        if y < 0.0 && y.abs() > x.abs() {
            add_action(ControllerInput::DirectionDown, input_map, result);
            gamepad.last_input = now;
        }
        if x > 0.0 && x.abs() > y.abs() {
            add_action(ControllerInput::DirectionRight, input_map, result);
            gamepad.last_input = now;
        }
    }
}

fn add_action(
    input: ControllerInput,
    input_map: &InputMap<KeyCode, ControllerInput, Actions>,
    result: &mut std::collections::HashSet<Actions>,
) {
    if let Some(action) = input_map.get(None, Some(input)) {
        result.insert(action.clone());
    } else {
        result.insert(Actions::KeyPress(InputType::Gamepad(input)));
    }
}
