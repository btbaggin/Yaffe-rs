#![allow(dead_code)] 

use winapi::shared::minwindef::{WORD, BYTE, DWORD};
use winapi::um::winnt::SHORT;
use winapi::shared::winerror::{ERROR_DEVICE_NOT_CONNECTED, ERROR_SUCCESS};
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryW};
use std::time::Instant;

pub enum XInputError {
    InvalidControllerID,
    DeviceNotConnected,
    UnknownError(u32),
}

pub const CONTROLLER_DPAD_UP: u16 = 0x0001;
pub const CONTROLLER_DPAD_DOWN: u16 = 0x0002;
pub const CONTROLLER_DPAD_LEFT: u16 = 0x0004;
pub const CONTROLLER_DPAD_RIGHT: u16 = 0x0008;
pub const CONTROLLER_START: u16 = 0x0010;
pub const CONTROLLER_BACK: u16 = 0x0020;
pub const CONTROLLER_LEFT_THUMB: u16 = 0x0040;
pub const CONTROLLER_RIGHT_THUMB: u16 = 0x0080;
pub const CONTROLLER_LEFT_SHOULDER: u16 = 0x0100;
pub const CONTROLLER_RIGHT_SHOULDER: u16 = 0x0200;
pub const CONTROLLER_GUIDE: u16 = 0x0400;
pub const CONTROLLER_A: u16 = 0x1000;
pub const CONTROLLER_B: u16 = 0x2000;
pub const CONTROLLER_X: u16 = 0x4000;
pub const CONTROLLER_Y: u16 = 0x8000;

pub const CONTROLLER_UP: u16 = 0x1001;
pub const CONTROLLER_DOWN: u16 = 0x2001;
pub const CONTROLLER_LEFT: u16 = 0x4001;
pub const CONTROLLER_RIGHT: u16 = 0x8001;

const XINPUT_GAMEPAD_DEADZONE: i32 = 7849;

type XInputGetStateFunc = unsafe extern "system" fn(DWORD, *mut XInputGamepadEx) -> DWORD;

#[repr(C)] #[derive(Default, Copy, Clone)]
pub struct XInputGamepadEx {
	event_count: DWORD,
	w_buttons: WORD,
	b_left_trigger: BYTE,
	b_right_trigger: BYTE,
	s_thumb_lx: SHORT,
	s_thumb_ly: SHORT,
	s_thumb_rx: SHORT,
	s_thumb_ry: SHORT,
}

pub struct XInput {
    get_state: XInputGetStateFunc,
    previous_state: XInputGamepadEx,
    current_state: XInputGamepadEx,
    last_stick_time: Instant,
    last_button_time: Instant,
}

impl XInput {
    pub fn update(&mut self, user_index: u32) -> Result<(), XInputError> {
        if user_index >= 4 {
            Err(XInputError::InvalidControllerID)
        } else {
            let mut output: XInputGamepadEx = unsafe { ::std::mem::zeroed() };
            let return_status = unsafe { (self.get_state)(user_index as DWORD, &mut output) };
            match return_status {
                ERROR_SUCCESS => {
                    self.previous_state = self.current_state;
                    self.current_state = output;
                    return Ok(());
                }
                ERROR_DEVICE_NOT_CONNECTED => Err(XInputError::DeviceNotConnected),
                s => { Err(XInputError::UnknownError(s)) }
            }
        }
    }

    fn is_pressed(&self, button: u16) -> bool {
        self.current_state.w_buttons & button != 0 &&
        self.previous_state.w_buttons & button == 0
    }

    fn stick_is_pressed(&self, direction: u16) -> bool {
        let x = self.current_state.s_thumb_lx as i32;
        let y = self.current_state.s_thumb_ly as i32;
        if (x * x) + (y * y) < XINPUT_GAMEPAD_DEADZONE * XINPUT_GAMEPAD_DEADZONE { return false; }
    
        match direction {
            CONTROLLER_LEFT => x < 0 && i32::abs(x) > i32::abs(y),
            CONTROLLER_UP => y > 0 && i32::abs(y) > i32::abs(x),
            CONTROLLER_DOWN => y < 0 && i32::abs(y) > i32::abs(x),
            CONTROLLER_RIGHT => x > 0 && i32::abs(x) > i32::abs(y),
            _ => panic!("unknown direction")
        }
    }

    pub fn get_actions(&mut self) -> Vec<u16> {
        let mut result = Vec::new();

        let now = Instant::now();
        if (now - self.last_button_time).as_millis() > 100 {
            for action in [CONTROLLER_DPAD_UP, CONTROLLER_DPAD_DOWN, CONTROLLER_DPAD_LEFT, CONTROLLER_DPAD_RIGHT, CONTROLLER_START, CONTROLLER_BACK,
                        CONTROLLER_LEFT_THUMB, CONTROLLER_RIGHT_THUMB, CONTROLLER_LEFT_SHOULDER, CONTROLLER_RIGHT_SHOULDER, CONTROLLER_GUIDE,
                        CONTROLLER_A, CONTROLLER_B, CONTROLLER_X, CONTROLLER_Y].iter() {
                if self.is_pressed(*action) { 
                    result.push(*action); 
                    self.last_button_time = now;
                }
            }
        }

        if (now - self.last_stick_time).as_millis() > 100 {
            for action in [CONTROLLER_UP, CONTROLLER_DOWN, CONTROLLER_LEFT, CONTROLLER_RIGHT].iter() {
                if self.stick_is_pressed(*action) { 
                    result.push(*action); 
                    self.last_stick_time = now;
                }
            }
        }

        result
    }
}

pub fn load_xinput() -> Option<XInput> {
    for lib_name in ["xinput1_4.dll", "xinput1_3.dll", "xinput1_2.dll", "xinput1_1.dll"].iter() {
        let handle = load(lib_name);
        if handle.is_some() { return handle; }
    }
    None
}

fn load<S: AsRef<str>>(s: S) -> Option<XInput> {
    fn wide_null<S: AsRef<str>>(s: S) -> [u16; ::winapi::shared::minwindef::MAX_PATH] {
        let mut output: [u16; ::winapi::shared::minwindef::MAX_PATH] = [0; ::winapi::shared::minwindef::MAX_PATH];
        let mut i = 0;
        for u in s.as_ref().encode_utf16() {
            if i == output.len() - 1 { break; } 
            else { output[i] = u; }
            i += 1;
        }
        output[i] = 0;
        output
      }

    let lib_name = wide_null(s);
    // It's always safe to call `LoadLibraryW`, the worst that can happen is
    // that we get a null pointer back.
    let xinput_handle = unsafe { LoadLibraryW(lib_name.as_ptr()) };
    if xinput_handle.is_null() { return None; }

    let mut opt_xinput_get_state = None;
    // using transmute is so dodgy we'll put that in its own unsafe block.
    unsafe {
        let get_state_ptr = GetProcAddress(xinput_handle, 100 as *mut i8);
        // let get_state_ptr = GetProcAddress(xinput_handle, b"XInputGetState\0".as_ptr() as *mut i8);
        if !get_state_ptr.is_null() {
            opt_xinput_get_state = Some(::std::mem::transmute(get_state_ptr));
        }
    }

    unsafe { FreeLibrary(xinput_handle); }

    Some(XInput { get_state: opt_xinput_get_state.unwrap(), 
                  current_state: XInputGamepadEx::default(), 
                  previous_state: XInputGamepadEx::default(),
                  last_stick_time: Instant::now(), 
                  last_button_time: Instant::now() })
}