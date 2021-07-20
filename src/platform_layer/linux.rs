use super::{ShutdownResult, StartupResult};
use std::io::{Error, ErrorKind};
use glutin::event::VirtualKeyCode;
use std::os::unix::io::AsRawFd;
// use x11::xlib::{XInternAtom, XLookupNone};
use std::process::Command;

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

        if self.check_for_joystick("/dev/input/js0") { self.get_joystick(); }
        Ok(())
    }

    fn get_gamepad(&mut self) -> Vec<super::ControllerInput> {
        panic!()
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