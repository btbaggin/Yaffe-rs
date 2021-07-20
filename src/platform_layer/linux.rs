use super::{ShutdownResult, StartupResult, VolumeResult};
use std::io::{Error, ErrorKind};
use std::os::unix::io::AsRawFd;
// use x11::xlib::{XInternAtom, XLookupNone};
use std::process::Command;
use std::io::Read;
use std::time::Instant;

pub(super) fn get_run_at_startup(_: &str) -> StartupResult<bool> {
    panic!()
}

pub(super) fn set_run_at_startup(_: &str, _: bool) -> StartupResult<()> {
    panic!()
}

pub(super) fn shutdown() -> ShutdownResult {
    let mut cmd = Command::new("shutdown");
    cmd.args(&["-h", "now"]);
    match cmd.output() {
        Ok(output) => {
            if output.status.success() {
                return Ok(());
            }
            Err(Error::new(ErrorKind::Other, String::from_utf8(output.stderr).unwrap()))
        }
        Err(error) => Err(error),
    }
}

const XINPUT_GAMEPAD_START: u16 = 0x0010;
const XINPUT_GAMEPAD_BACK: u16 = 0x0020;
const CONTROLLER_GUIDE: u16 = 0x0400;
const XINPUT_GAMEPAD_A: u16 = 0x1000;
const XINPUT_GAMEPAD_B: u16 = 0x2000;
const XINPUT_GAMEPAD_X: u16 = 0x4000;
const XINPUT_GAMEPAD_Y: u16 = 0x8000;
const XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE: u16 = 7849;

#[derive(Default, Clone, Copy)]
struct LinuxInputState {
    w_buttons: u16,
	s_thumb_lx: i16,
	s_thumb_ly: i16,
	s_thumb_rx: i16,
	s_thumb_ry: i16,
}

struct LinuxInput {
    joystick: Option<std::fs::File>,
    previous_state: LinuxInputState,
    current_state: LinuxInputState,
    last_stick_time: Instant,
    last_button_time: Instant,
}

#[repr(C, packed)]
struct JsEvent {
 	time: u32,	/* event timestamp in milliseconds */
    value: i16,	/* value */
 	event_type: u8,	/* event type */
 	number: u8,	/* axis/button number */
}

const JS_EVENT_BUTTON: u8 = 0x01;	/* button pressed/released */
const JS_EVENT_AXIS: u8 = 0x02;	/* joystick moved */
const JS_EVENT_INIT: u8 = 0x80;	/* initial state of device */

impl crate::input::PlatformGamepad for LinuxInput {
    fn update(&mut self, user_index: u32) -> Result<(), u32> {
        self.previous_state = self.current_state;

        let path = format!("/dev/input/js{}", user_index);
        if self.check_for_joystick(&path) { self.get_joystick(); }
        Ok(())
    }

    fn get_gamepad(&mut self) -> Vec<super::ControllerInput> {
        fn is_pressed(input: &LinuxInput, button: u16) -> bool {
			input.current_state.w_buttons & button != 0 && input.previous_state.w_buttons & button == 0
		}
		let mut result = Vec::new();

		let now = Instant::now();
		if (now - self.last_button_time).as_millis() > 100 {
			let count = result.len();
			if is_pressed(self, XINPUT_GAMEPAD_START) { result.push(super::ControllerInput::ButtonStart); }
			if is_pressed(self, XINPUT_GAMEPAD_BACK) { result.push(super::ControllerInput::ButtonBack); }
			if is_pressed(self, CONTROLLER_GUIDE) { result.push(super::ControllerInput::ButtonGuide); }
			if is_pressed(self, XINPUT_GAMEPAD_A) { result.push(super::ControllerInput::ButtonSouth); }
			if is_pressed(self, XINPUT_GAMEPAD_B) { result.push(super::ControllerInput::ButtonEast); }
			if is_pressed(self, XINPUT_GAMEPAD_X) { result.push(super::ControllerInput::ButtonWest); }
			if is_pressed(self, XINPUT_GAMEPAD_Y) { result.push(super::ControllerInput::ButtonNorth); }
			if result.len() > count { self.last_button_time = now; }
		}

		let x = self.current_state.s_thumb_lx as i32;
		let y = self.current_state.s_thumb_ly as i32;
		if (x * x) + (y * y) > XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE as i32 * XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE as i32 { 
			if (now - self.last_stick_time).as_millis() > 100 {
				let count = result.len();
				if x < 0 && i32::abs(x) > i32::abs(y) { result.push(super::ControllerInput::DirectionLeft); }
				if y > 0 && i32::abs(y) > i32::abs(x) { result.push(super::ControllerInput::DirectionUp); }
				if y < 0 && i32::abs(y) > i32::abs(x) { result.push(super::ControllerInput::DirectionDown); }
				if x > 0 && i32::abs(x) > i32::abs(y) { result.push(super::ControllerInput::DirectionRight); }
				if result.len() > count { self.last_stick_time = now; }
			}
		}

        result
    }
}

impl LinuxInput {
    fn check_for_joystick(&mut self, path: &str) -> bool {
        // Opens device in blocking mode.
        if let None = self.joystick {
            match std::fs::File::open(path) {
                Ok(file) => self.joystick = Some(file),
                Err(_) => return false,
            } 

            // Changes into a NON-BLOCKING-MODE.
            unsafe { libc::fcntl(self.joystick.as_ref().unwrap().as_raw_fd(), libc::F_SETFL, libc::O_NONBLOCK); }
        }
        return true;
    }

    fn get_joystick(&mut self) -> LinuxInputState {
        let mut state = LinuxInputState::default();
        if let Some(mut joystick) = self.joystick.as_ref() {

            /* read all events from the driver stack! */
            let mut buffer = Vec::with_capacity(std::mem::size_of::<JsEvent>());
            while joystick.read_exact(&mut buffer).is_ok() {

                let s: JsEvent = unsafe { std::ptr::read(buffer.as_ptr() as *const _) };
                match s.event_type & !JS_EVENT_INIT {
                    JS_EVENT_AXIS => {
                        match s.number {
                            0 => state.s_thumb_lx = s.value,
                            2 => { /* left trigger */ },
                            1 => state.s_thumb_ly = -s.value,
                            3 => state.s_thumb_rx = s.value,
                            4 => state.s_thumb_ry = -s.value,
                            5 => { /* right trigger */ },
                            _ => assert!(false, "Unknown joystick axis"),
                        }
                    },
                    JS_EVENT_BUTTON => {
                        let button = match s.number {
                            0 => XINPUT_GAMEPAD_A,
                            1 => XINPUT_GAMEPAD_B,
                            2 => XINPUT_GAMEPAD_X,
                            3 => XINPUT_GAMEPAD_Y,
                            4 => { /* left shoulder */ 0 },
                            5 => { /* right shoulder */ 0 },
                            6 => XINPUT_GAMEPAD_BACK,
                            7 => XINPUT_GAMEPAD_START,
                            8 => CONTROLLER_GUIDE,
                            9 => { /* left thumb */ 0 },
                            10 => { /* right thumb */ 0 },
                            _ => { assert!(false, "Unknown joystick button"); 0 },
                        };

                        if s.value != 0 { /* up */ state.w_buttons |= button }
                        else { /* down */ state.w_buttons ^= button; }
                    },
                    _ => assert!(false, "Unknown joystick event"),
                }
            }
        }
        state
    }
}

pub fn initialize_gamepad() -> Result<impl crate::input::PlatformGamepad, i32> {
    Ok(LinuxInput {
        joystick: None,
        previous_state: LinuxInputState::default(),
        current_state: LinuxInputState::default(),
        last_stick_time: Instant::now(),
        last_button_time: Instant::now(),
    })
}

pub(super) fn get_clipboard() -> Option<String> {
        panic!();

        // char* c = 0;
        // let UTF8 = XInternAtom(x11::xlib::Display::, "UTF8_STRING", True);
        // if UTF8 != XLookupNone { c = XPasteType(UTF8, pState->form->platform, UTF8); }
        // if !c { c = XPasteType(XA_STRING, pState->form->platform, UTF8); }
}

pub(super) fn get_and_update_volume(_: f32) -> VolumeResult<f32> {
    panic!();
}
