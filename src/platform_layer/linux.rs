use super::{ShutdownResult, StartupResult};
use std::io::{Error, ErrorKind};
use glutin::event::VirtualKeyCode;
use std::os::unix::io::AsRawFd;
use x11::xlib::{XInternAtom, XLookupNone};
use std::process::Command;
use x11::keysym::*;

pub(super) fn get_run_at_startup(task: &str) -> StartupResult<bool> {
    panic!()
}

pub(super) fn set_run_at_startup(task: &str, value: bool) -> StartupResult<()> {
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

struct LinuxInput {
    joystick: Option<std::fs::File>,
    previous_state: [i8; 32],
    current_state: [i8; 32],
}

impl crate::input::PlatformInput for LinuxInput {
    fn update(&mut self, user_index: u32) -> Result<(), u32> {
        self.previous_state = self.current_state;

        unsafe { x11::xlib::XQueryKeymap(std::ptr::null_mut(), self.current_state.as_mut_ptr()); }

        if self.check_for_joystick("/dev/input/js0") { self.get_joystick(); }
        Ok(())
    }

    fn get_gamepad(&mut self) -> Vec<super::ControllerInput> {
        panic!()
    }

	fn get_keyboard(&mut self) -> Vec<(VirtualKeyCode, Option<char>)> {
        for i in 0..255 {
            if self.current_state[i >> 3] & (1 << (i & 7)) != 0 &&
               self.current_state[i >> 3] & (1 << (i & 7)) == 0 {
                #[allow(non_upper_case_globals)]
                   let key = match unsafe { x11::xlib::XKeycodeToKeysym(std::ptr::null_mut(), i as u8, 0) } as u32 {
                    XK_BackSpace => Some(VirtualKeyCode::Back),
                    XK_Tab => Some(VirtualKeyCode::Tab),
                    XK_Return => Some(VirtualKeyCode::Return),
                    XK_Escape => Some(VirtualKeyCode::Escape),
                    XK_Delete => Some(VirtualKeyCode::Delete),
                    XK_Home => Some(VirtualKeyCode::Home),
                    XK_Left => Some(VirtualKeyCode::Left),
                    XK_Up => Some(VirtualKeyCode::Up),
                    XK_Right => Some(VirtualKeyCode::Right),
                    XK_Down => Some(VirtualKeyCode::Down),
                    XK_Page_Up => Some(VirtualKeyCode::PageUp),
                    XK_Page_Down => Some(VirtualKeyCode::PageDown),
                    XK_End => Some(VirtualKeyCode::End),
                    XK_Insert => Some(VirtualKeyCode::Insert),
                    XK_KP_Home => Some(VirtualKeyCode::Home),
                    XK_KP_Left => Some(VirtualKeyCode::Left),
                    XK_KP_Up => Some(VirtualKeyCode::Up),
                    XK_KP_Right => Some(VirtualKeyCode::Right),
                    XK_KP_Down => Some(VirtualKeyCode::Down),
                    XK_KP_Page_Up => Some(VirtualKeyCode::PageUp),
                    XK_KP_Page_Down => Some(VirtualKeyCode::PageDown),
                    XK_KP_End => Some(VirtualKeyCode::End),
                    XK_KP_Insert => Some(VirtualKeyCode::Insert),
                    XK_KP_Delete => Some(VirtualKeyCode::Delete),
                    XK_KP_Equal => Some(VirtualKeyCode::NumpadEquals),
                    XK_KP_Multiply => Some(VirtualKeyCode::NumpadMultiply),
                    XK_KP_Add => Some(VirtualKeyCode::NumpadAdd),
                    XK_KP_Subtract => Some(VirtualKeyCode::NumpadSubtract),
                    XK_KP_Decimal => Some(VirtualKeyCode::NumpadDecimal),
                    XK_KP_Divide => Some(VirtualKeyCode::NumpadDivide),
                    XK_KP_0 => Some(VirtualKeyCode::Numpad0),
                    XK_KP_1 => Some(VirtualKeyCode::Numpad1),
                    XK_KP_2 => Some(VirtualKeyCode::Numpad2),
                    XK_KP_3 => Some(VirtualKeyCode::Numpad3),
                    XK_KP_4 => Some(VirtualKeyCode::Numpad4),
                    XK_KP_5 => Some(VirtualKeyCode::Numpad5),
                    XK_KP_6 => Some(VirtualKeyCode::Numpad6),
                    XK_KP_7 => Some(VirtualKeyCode::Numpad7),
                    XK_KP_8 => Some(VirtualKeyCode::Numpad8),
                    XK_KP_9 => Some(VirtualKeyCode::Numpad9),
                    XK_F1 => Some(VirtualKeyCode::F1),
                    XK_F2 => Some(VirtualKeyCode::F2),
                    XK_F3 => Some(VirtualKeyCode::F3),
                    XK_F4 => Some(VirtualKeyCode::F4),
                    XK_F5 => Some(VirtualKeyCode::F5),
                    XK_F6 => Some(VirtualKeyCode::F6),
                    XK_F7 => Some(VirtualKeyCode::F7),
                    XK_F8 => Some(VirtualKeyCode::F8),
                    XK_F9 => Some(VirtualKeyCode::F9),
                    XK_F10 => Some(VirtualKeyCode::F10),
                    XK_F11 => Some(VirtualKeyCode::F11),
                    XK_F12 => Some(VirtualKeyCode::F12),
                    XK_Shift_L => Some(VirtualKeyCode::LShift),
                    XK_Shift_R => Some(VirtualKeyCode::RShift),
                    XK_Control_L => Some(VirtualKeyCode::LControl),
                    XK_Control_R => Some(VirtualKeyCode::RControl),
                    XK_Alt_L => Some(VirtualKeyCode::LAlt),
                    XK_Alt_R => Some(VirtualKeyCode::RAlt),
                    XK_ISO_Left_Tab => Some(VirtualKeyCode::Tab),
                    XK_space => Some(VirtualKeyCode::Space),
                    XK_apostrophe => Some(VirtualKeyCode::Apostrophe),
                    XK_asterisk => Some(VirtualKeyCode::Asterisk),
                    XK_plus => Some(VirtualKeyCode::Plus),
                    XK_comma => Some(VirtualKeyCode::Comma),
                    XK_minus => Some(VirtualKeyCode::Minus),
                    XK_period => Some(VirtualKeyCode::Period),
                    XK_slash => Some(VirtualKeyCode::Slash),
                    XK_0 => Some(VirtualKeyCode::Key0),
                    XK_1 => Some(VirtualKeyCode::Key1),
                    XK_2 => Some(VirtualKeyCode::Key2),
                    XK_3 => Some(VirtualKeyCode::Key3),
                    XK_4 => Some(VirtualKeyCode::Key4),
                    XK_5 => Some(VirtualKeyCode::Key5),
                    XK_6 => Some(VirtualKeyCode::Key6),
                    XK_7 => Some(VirtualKeyCode::Key7),
                    XK_8 => Some(VirtualKeyCode::Key8),
                    XK_9 => Some(VirtualKeyCode::Key9),
                    XK_colon => Some(VirtualKeyCode::Colon),
                    XK_semicolon => Some(VirtualKeyCode::Semicolon),
                    XK_equal => Some(VirtualKeyCode::Equals),
                    XK_at => Some(VirtualKeyCode::At),
                    XK_A => Some(VirtualKeyCode::A),
                    XK_B => Some(VirtualKeyCode::B),
                    XK_C => Some(VirtualKeyCode::C),
                    XK_D => Some(VirtualKeyCode::D),
                    XK_E => Some(VirtualKeyCode::E),
                    XK_F => Some(VirtualKeyCode::F),
                    XK_G => Some(VirtualKeyCode::G),
                    XK_H => Some(VirtualKeyCode::H),
                    XK_I => Some(VirtualKeyCode::I),
                    XK_J => Some(VirtualKeyCode::J),
                    XK_K => Some(VirtualKeyCode::K),
                    XK_L => Some(VirtualKeyCode::L),
                    XK_M => Some(VirtualKeyCode::M),
                    XK_N => Some(VirtualKeyCode::N),
                    XK_O => Some(VirtualKeyCode::O),
                    XK_P => Some(VirtualKeyCode::P),
                    XK_Q => Some(VirtualKeyCode::Q),
                    XK_R => Some(VirtualKeyCode::R),
                    XK_S => Some(VirtualKeyCode::S),
                    XK_T => Some(VirtualKeyCode::T),
                    XK_U => Some(VirtualKeyCode::U),
                    XK_V => Some(VirtualKeyCode::V),
                    XK_W => Some(VirtualKeyCode::W),
                    XK_X => Some(VirtualKeyCode::X),
                    XK_Y => Some(VirtualKeyCode::Y),
                    XK_Z => Some(VirtualKeyCode::Z),
                    XK_bracketleft => Some(VirtualKeyCode::LBracket),
                    XK_backslash => Some(VirtualKeyCode::Backslash),
                    XK_bracketright => Some(VirtualKeyCode::RBracket),
                    XK_grave => Some(VirtualKeyCode::Grave),
                    XK_a => Some(VirtualKeyCode::A),
                    XK_b => Some(VirtualKeyCode::B),
                    XK_c => Some(VirtualKeyCode::C),
                    XK_d => Some(VirtualKeyCode::D),
                    XK_e => Some(VirtualKeyCode::E),
                    XK_f => Some(VirtualKeyCode::F),
                    XK_g => Some(VirtualKeyCode::G),
                    XK_h => Some(VirtualKeyCode::H),
                    XK_i => Some(VirtualKeyCode::I),
                    XK_j => Some(VirtualKeyCode::J),
                    XK_k => Some(VirtualKeyCode::K),
                    XK_l => Some(VirtualKeyCode::L),
                    XK_m => Some(VirtualKeyCode::M),
                    XK_n => Some(VirtualKeyCode::N),
                    XK_o => Some(VirtualKeyCode::O),
                    XK_p => Some(VirtualKeyCode::P),
                    XK_q => Some(VirtualKeyCode::Q),
                    XK_r => Some(VirtualKeyCode::R),
                    XK_s => Some(VirtualKeyCode::S),
                    XK_t => Some(VirtualKeyCode::T),
                    XK_u => Some(VirtualKeyCode::U),
                    XK_v => Some(VirtualKeyCode::V),
                    XK_w => Some(VirtualKeyCode::W),
                    XK_x => Some(VirtualKeyCode::X),
                    XK_y => Some(VirtualKeyCode::Y),
                    XK_z => Some(VirtualKeyCode::Z),
                    _ => None
                };
        
                if let Some(k) = key {
                    let c = unsafe { i as u8 as char };
                    //result.push((k, c));
                }
            }
        }
        panic!()
    }

    fn get_modifiers(&mut self) -> glutin::event::ModifiersState {
        use glutin::event::ModifiersState;
        let result = ModifiersState::empty();
        panic!();
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

    fn get_joystick(&self) -> u32 {
        let buttons = 0;
        // struct js_event jsEvent;

        // /* read all events from the driver stack! */
        // while (read(pJoystick, &jsEvent, sizeof(struct js_event)) > 0) 
        // {
        //     switch (e.type & ~JS_EVENT_INIT) 
                // {
                //     case JS_EVENT_AXIS:
                //         switch (e.number) 
                //         {
                //             case 0:	pInput->left_stick.X = e.value; break;
                //             case 1:	pInput->left_stick.Y = -e.value; break;
                //             case 3:	pInput->right_stick.X = e.value; break;
                //             case 4:	pInput->right_stick.Y = -e.value; break;
                //             //2 is left trigger
                //             //5 is right trigger
                //         }
                //         break;

                //     case JS_EVENT_BUTTON:
                //         int button;
                //         switch(e.number)
                //         {
                //             case 0: button = CONTROLLER_A; break;
                //             case 1: button = CONTROLLER_B; break;
                //             case 2: button = CONTROLLER_X; break;
                //             case 3: button = CONTROLLER_Y; break;
                //             case 4: button = CONTROLLER_LEFT_SHOULDER; break;
                //             case 5: button = CONTROLLER_RIGHT_SHOULDER; break;
                //             case 6: button = CONTROLLER_BACK; break;
                //             case 7: button = CONTROLLER_START; break;
                //             case 8: button = CONTROLLER_GUIDE; break; 
                //             case 9: button = CONTROLLER_LEFT_THUMB; break;
                //             case 10: button = CONTROLLER_RIGHT_THUMB; break;
                //             default: button = 0; break;
                //         }

                //         if (e.value) pInput->current_controller_buttons |= button;
                //         else pInput->current_controller_buttons ^= button;
                //         break;

                //     default:
                //         break;
                // }
        // }
        buttons
    }
}

pub fn initialize_input() -> Result<impl crate::input::PlatformInput, i32> {
    Ok(LinuxInput {
        joystick: None,
        previous_state: [0; 32],
        current_state: [0; 32],
    })
}

pub(super) fn get_clipboard() -> Option<String> {
    unsafe {
        panic!();

        // char* c = 0;
        // let UTF8 = XInternAtom(x11::xlib::Display::, "UTF8_STRING", True);
        // if UTF8 != XLookupNone { c = XPasteType(UTF8, pState->form->platform, UTF8); }
        // if !c { c = XPasteType(XA_STRING, pState->form->platform, UTF8); }
    }
}