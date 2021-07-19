use super::ShutdownResult;

pub(super) fn get_run_at_startup(task: &str) -> StartupResult<bool> {
}

pub(super) fn set_run_at_startup(task: &str, value: bool) -> StartupResult<()> {
}

fn(super) shutdown() -> ShutdownResult {
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
    joystick: i32,
}

impl crate::input::PlatformInput for LinuxInput {
    fn update(&mut self, user_index: u32) -> Result<(), u32> {
        // self.previous_state = self.current_state;

        // XQueryKeymap(pDisplay->display, pInput->current_keyboard_state);

        // pInput->current_keyboard_state[INPUT_SIZE - 1] |= pPress;
        // pInput->current_keyboard_state[INPUT_SIZE - 1] &= ~pRelease;

        if self.check_for_joystick("/dev/input/js0") { self.get_joystick() }
    }

    fn get_gamepad(&mut self) -> Vec<super::ControllerInput> {
    }

	fn get_keyboard(&mut self) -> Vec<(VirtualKeyCode, Option<char>)> {
    }

    fn get_modifiers(&mut self) -> glutin::event::ModifiersState {
    }
}

impl LinuxInput {
    fn check_for_joystick(&mut self) -> bool {
        // Opens device in blocking mode.
        if (self.joystick == -1)
        {
            self.joystick = open(pName, O_RDONLY);
            if self.joystick == -1 { return false; }

            // Changes into a NON-BLOCKING-MODE.
            fcntl(self.joystick, F_SETFL, O_NONBLOCK);
        }
        return true;
    }

    fn get_joystick(&self) {
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
    }
}

pub fn initialize_input() -> Result<impl crate::input::PlatformInput, i32> {
}

pub(super) get_input() -> bool { 
	true
}

pub(super) get_clipboard() -> Option<String> {
    // char* c = 0;
	// Atom UTF8 = XInternAtom(pState->form->platform->display, "UTF8_STRING", True);
	// if(UTF8 != None) c = XPasteType(UTF8, pState->form->platform, UTF8);
	// if(!c) c = XPasteType(XA_STRING, pState->form->platform, UTF8);
	// return c;
}