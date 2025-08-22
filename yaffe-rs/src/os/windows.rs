use super::{PlatformError, PlatformResult};
use crate::input::ControllerInput;
use windows::{
    core::*,
    Gaming::Input::{Gamepad, GamepadButtons, GamepadReading},
    Win32::{
        Foundation::{HANDLE, LUID, VARIANT_TRUE},
        Media::Audio::{eConsole, eRender, Endpoints::IAudioEndpointVolume, IMMDeviceEnumerator, MMDeviceEnumerator},
        Security::{
            AdjustTokenPrivileges, LookupPrivilegeValueW, SE_SHUTDOWN_NAME, TOKEN_ADJUST_PRIVILEGES, TOKEN_PRIVILEGES,
            TOKEN_QUERY,
        },
        System::{
            Com::{
                CoCreateInstance, CoInitializeEx, CoUninitialize, CLSCTX_ALL, CLSCTX_INPROC_SERVER, COINIT,
                COINIT_APARTMENTTHREADED,
            },
            Shutdown::{ExitWindowsEx, EWX_FORCEIFHUNG, EWX_SHUTDOWN, SHUTDOWN_REASON},
            TaskScheduler::{
                IExecAction, ITaskService, TaskScheduler, TASK_ACTION_EXEC, TASK_LOGON_TYPE, TASK_RUNLEVEL_HIGHEST,
                TASK_TRIGGER_LOGON,
            },
            Threading::{GetCurrentProcess, OpenProcessToken},
            Variant::VARIANT,
        },
    },
};

impl From<Error> for PlatformError {
    fn from(v: Error) -> Self { PlatformError::Other(format!("Error occurrted ({}): {})", v.code(), v.message())) }
}

pub fn get_run_at_startup(task_name: &str) -> PlatformResult<bool> {
    unsafe {
        let _com_guard = ComGuard::new(Some(COINIT_APARTMENTTHREADED))?;

        // Create task service
        let task_service: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;
        task_service.Connect(&VARIANT::default(), &VARIANT::default(), &VARIANT::default(), &VARIANT::default())?;

        let root_folder = task_service.GetFolder(&BSTR::from("\\"))?;

        // Try to get the task
        let result: Result<bool> = match root_folder.GetTask(&BSTR::from(task_name)) {
            Ok(task) => {
                // Task exists, check if it's enabled
                let enabled = task.Enabled()?;
                Ok(enabled.as_bool())
            }
            Err(_) => Ok(false), // Task doesn't exist
        };

        Ok(result?)
    }
}

pub fn set_run_at_startup(task_name: &str, value: bool) -> PlatformResult<()> {
    unsafe {
        let _com_guard = ComGuard::new(Some(COINIT_APARTMENTTHREADED))?;

        if value {
            create_startup_task(task_name)?
        } else {
            delete_startup_task(task_name)?
        };

        Ok(())
    }
}

unsafe fn create_startup_task(task_name: &str) -> Result<()> {
    // Get current executable path and working directory
    let exe_path = std::env::current_exe()?;
    let working_dir = exe_path.parent().unwrap();

    // Create task service
    let task_service: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;
    task_service.Connect(&VARIANT::default(), &VARIANT::default(), &VARIANT::default(), &VARIANT::default())?;

    let root_folder = task_service.GetFolder(&BSTR::from("\\"))?;

    // Delete existing task if it exists (ignore errors)
    let _ = root_folder.DeleteTask(&BSTR::from(task_name), 0);

    // Create new task definition
    let task_def = task_service.NewTask(0)?;

    // Set principal to run with highest privileges
    let principal = task_def.Principal()?;
    principal.SetRunLevel(TASK_RUNLEVEL_HIGHEST)?;

    // Configure task settings
    let settings = task_def.Settings()?;
    settings.SetStartWhenAvailable(VARIANT_TRUE)?;
    settings.SetEnabled(VARIANT_TRUE)?;

    // Create logon trigger
    let triggers = task_def.Triggers()?;
    let trigger = triggers.Create(TASK_TRIGGER_LOGON)?;
    trigger.SetEnabled(VARIANT_TRUE)?;

    // Create execution action
    let actions = task_def.Actions()?;
    let action = actions.Create(TASK_ACTION_EXEC)?;
    let exec_action: IExecAction = action.cast()?;

    // Set executable path and working directory
    exec_action.SetPath(&BSTR::from(exe_path.to_string_lossy().as_ref()))?;
    exec_action.SetWorkingDirectory(&BSTR::from(working_dir.to_string_lossy().as_ref()))?;

    // Register the task
    root_folder.RegisterTaskDefinition(
        &BSTR::from(task_name),
        &task_def,
        6, // TASK_CREATE_OR_UPDATE
        &VARIANT::default(),
        &VARIANT::default(),
        TASK_LOGON_TYPE(0), // TASK_LOGON_NONE
        &VARIANT::default(),
    )?;

    Ok(())
}

unsafe fn delete_startup_task(task_name: &str) -> Result<()> {
    // Create task service
    let task_service: ITaskService = CoCreateInstance(&TaskScheduler, None, CLSCTX_INPROC_SERVER)?;

    // Connect to task service
    task_service.Connect(&VARIANT::default(), &VARIANT::default(), &VARIANT::default(), &VARIANT::default())?;

    // Get root folder
    let root_folder = task_service.GetFolder(&BSTR::from("\\"))?;

    // Delete the task (ignore error if task doesn't exist)
    match root_folder.DeleteTask(&BSTR::from(task_name), 0) {
        Ok(_) => Ok(()),
        Err(_) => Ok(()), // Task probably didn't exist, which is fine
    }
}

pub fn lib_ext() -> &'static str { "dll" }

pub fn app_ext() -> &'static str { "exe" }

pub(super) fn shutdown() -> PlatformResult<()> {
    unsafe {
        let mut token_handle = HANDLE::default();
        OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token_handle)?;

        let mut luid = LUID::default();
        LookupPrivilegeValueW(None, SE_SHUTDOWN_NAME, &mut luid)?;

        let token_privileges = TOKEN_PRIVILEGES {
            PrivilegeCount: 1,
            Privileges: [windows::Win32::Security::LUID_AND_ATTRIBUTES {
                Luid: luid,
                Attributes: windows::Win32::Security::SE_PRIVILEGE_ENABLED,
            }],
        };
        AdjustTokenPrivileges(token_handle, false, Some(&token_privileges), 0, None, None)?;

        ExitWindowsEx(EWX_SHUTDOWN | EWX_FORCEIFHUNG, SHUTDOWN_REASON(0))?;

        if !token_handle.is_invalid() {
            let _ = windows::Win32::Foundation::CloseHandle(token_handle);
        }
    }

    Ok(())
}

pub fn initialize_gamepad() -> Result<impl crate::input::PlatformGamepad> { Ok(WindowsInput::new(0)) }

struct WindowsInput {
    gamepad: Option<Gamepad>,
    current_reading: Option<GamepadReading>,
    previous_reading: Option<GamepadReading>,
    controller_id: usize,
    deadzone_threshold: f64,
    input_map: std::collections::HashMap<ControllerInput, GamepadButtons>,
}
impl WindowsInput {
    pub fn new(controller_id: usize) -> Self {
        // let now = Instant::now();

        let input_map = std::collections::HashMap::from([
            (ControllerInput::ButtonBack, GamepadButtons::View),
            (ControllerInput::ButtonStart, GamepadButtons::Menu),
            (ControllerInput::ButtonSouth, GamepadButtons::A),
            (ControllerInput::ButtonWest, GamepadButtons::X),
            (ControllerInput::ButtonNorth, GamepadButtons::Y),
            (ControllerInput::ButtonEast, GamepadButtons::B),
        ]);

        Self {
            gamepad: None,
            current_reading: None,
            previous_reading: None,
            controller_id,
            deadzone_threshold: 0.24, // Roughly equivalent to XInput deadzone
            input_map,
        }
    }

    fn is_gamepad_still_connected(&self) -> std::result::Result<bool, Box<dyn std::error::Error>> {
        if let Some(ref gamepad) = self.gamepad {
            // Try to get current reading to test if gamepad is still valid
            match gamepad.GetCurrentReading() {
                Ok(_) => Ok(true),
                Err(_) => Ok(false),
            }
        } else {
            Ok(false)
        }
    }

    fn find_gamepad(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        let gamepads = Gamepad::Gamepads()?;
        let count = gamepads.Size()? as usize;

        if self.controller_id < count {
            self.gamepad = Some(gamepads.GetAt(self.controller_id as u32)?);
        } else {
            self.gamepad = None;
            return Err(
                format!("Controller {} not found (only {} controllers available)", self.controller_id, count).into()
            );
        }

        Ok(())
    }

    /// Apply deadzone to thumbstick values
    fn apply_deadzone(&self, x: f64, y: f64) -> (f32, f32) {
        let magnitude = (x * x + y * y).sqrt();

        if magnitude < self.deadzone_threshold {
            (0.0, 0.0)
        } else {
            // Normalize and rescale outside deadzone
            let normalized_magnitude = (magnitude - self.deadzone_threshold) / (1.0 - self.deadzone_threshold);
            let normalized_magnitude = normalized_magnitude.min(1.0);

            let normalized_x = (x / magnitude) * normalized_magnitude;
            let normalized_y = (y / magnitude) * normalized_magnitude;

            (normalized_x as f32, normalized_y as f32)
        }
    }
}

impl crate::input::PlatformGamepad for WindowsInput {
    fn update(&mut self) -> std::result::Result<(), Box<dyn std::error::Error>> {
        // Check if we need to find our gamepad or if it's still valid
        if self.gamepad.is_none() || !self.is_gamepad_still_connected()? {
            self.find_gamepad()?;
        }

        // Read current state if we have a gamepad
        if let Some(ref gamepad) = self.gamepad {
            self.previous_reading = self.current_reading;
            self.current_reading = Some(gamepad.GetCurrentReading()?);
        }

        Ok(())
    }

    fn is_button_pressed(&self, button: ControllerInput) -> bool {
        let button = self.input_map.get(&button).expect("unknown button for controller");
        if let (Some(ref current), Some(ref previous)) = (&self.current_reading, &self.previous_reading) {
            current.Buttons.0 & button.0 != 0 && previous.Buttons.0 & button.0 == 0
        } else {
            false
        }
    }

    fn get_left_thumbstick(&self) -> (f32, f32) {
        if let Some(ref current) = self.current_reading {
            self.apply_deadzone(current.LeftThumbstickX, current.LeftThumbstickY)
        } else {
            (0.0, 0.0)
        }
    }
}

pub(super) fn get_volume() -> PlatformResult<f32> {
    unsafe {
        let _com_guard = ComGuard::new(None)?;

        // Create device enumerator
        let enumerator: IMMDeviceEnumerator =
            windows::Win32::System::Com::CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;
        let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;
        let volume_scalar = endpoint_volume.GetMasterVolumeLevelScalar()?;
        Ok(volume_scalar)
    }
}

pub(super) fn set_volume(amount: f32) -> PlatformResult<()> {
    unsafe {
        let _com_guard = ComGuard::new(None)?;

        // Create device enumerator
        let enumerator: IMMDeviceEnumerator =
            windows::Win32::System::Com::CoCreateInstance(&MMDeviceEnumerator, None, CLSCTX_ALL)?;

        // Get default audio endpoint (speakers)
        let device = enumerator.GetDefaultAudioEndpoint(eRender, eConsole)?;

        // Get the audio endpoint volume interface
        let endpoint_volume: IAudioEndpointVolume = device.Activate(CLSCTX_ALL, None)?;

        // Convert percentage to scalar (0.0 to 1.0)
        // Set the volume level
        endpoint_volume.SetMasterVolumeLevelScalar(amount, std::ptr::null())?;

        Ok(())
    }
}

struct ComGuard;
impl ComGuard {
    pub unsafe fn new(init: Option<COINIT>) -> PlatformResult<ComGuard> {
        let result = CoInitializeEx(None, init.unwrap_or(COINIT_APARTMENTTHREADED));
        if result.is_err() {
            Err(PlatformError::Other(String::new()))
        } else {
            Ok(ComGuard)
        }
    }
}
impl Drop for ComGuard {
    fn drop(&mut self) {
        unsafe {
            CoUninitialize();
        }
    }
}

pub fn sanitize_file(file: &str) -> String { file.replace(['\"', '*', '<', '>', '?', '\\', '/', ':'], "") }
