use winapi::um::taskschd::*;
use winapi::um::combaseapi::*;
use winapi::shared::wtypesbase::CLSCTX_INPROC_SERVER;
use winapi::shared::minwindef::{WORD, BYTE, DWORD, FALSE};
use winapi::shared::winerror::{ERROR_SUCCESS, ERROR_DEVICE_NOT_CONNECTED};
use winapi::shared::wtypes::{BSTR, VARIANT_TRUE};
use winapi::{Interface, Class};
use winapi::um::unknwnbase::IUnknown;
use winapi::um::processthreadsapi::{GetCurrentProcess, OpenProcessToken};
use winapi::um::reason::SHTDN_REASON_FLAG_PLANNED;
use winapi::um::securitybaseapi::AdjustTokenPrivileges;
use winapi::um::winbase::{LookupPrivilegeValueW, GlobalLock, GlobalUnlock};
use winapi::um::winnt::{
    HANDLE, SHORT, LPWSTR, SE_PRIVILEGE_ENABLED, SE_SHUTDOWN_NAME, TOKEN_ADJUST_PRIVILEGES,
    TOKEN_PRIVILEGES, TOKEN_QUERY,
};
use winapi::um::xinput::*;
use winapi::um::winuser::{
    ExitWindowsEx, EWX_FORCEIFHUNG, EWX_SHUTDOWN, OpenClipboard, GetClipboardData, CF_TEXT, CloseClipboard,
};
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryW};
use winapi::um::mmdeviceapi::{MMDeviceEnumerator, IMMDeviceEnumerator, IMMDevice, eRender, eConsole};
use winapi::um::endpointvolume::IAudioEndpointVolume;

use std::time::Instant;
use std::os::windows::ffi::OsStrExt;
use std::ops::Deref;
use std::convert::TryInto;
use super::{StartupResult, ShutdownResult, VolumeResult};

struct ComString { string: BSTR }
impl ComString {
	fn new(string: &str) -> ComString {
    	use std::iter::once;
		let wide: Vec<u16> = std::ffi::OsStr::new(string).encode_wide().chain(once(0)).collect();
		let string = unsafe { winapi::um::oleauto::SysAllocString(wide.as_ptr()) };
		ComString { string }
	}
}
impl Drop for ComString {
    fn drop(&mut self) {
		unsafe { winapi::um::oleauto::SysFreeString(self.string); }
    }
}
impl Deref for ComString {
    type Target = BSTR;
    fn deref(&self) -> &BSTR {
        &self.string 
    }
}


struct ComPtr<T: Interface> { ptr: *mut T }
impl<T: Interface> ComPtr<T> {
	fn default() -> ComPtr<T> {
		ComPtr { ptr: std::ptr::null_mut() }
	}
	fn as_mut(&mut self) -> *mut *mut T {
		&mut self.ptr as *mut *mut T
	}
	fn is_null(&self) -> bool {
		self.ptr.is_null()
	}
}
impl<T: Interface> Deref for ComPtr<T> {
    type Target = T;
    fn deref(&self) -> &T {
        unsafe { &mut *self.ptr }
    }
}
impl<T: Interface> Drop for ComPtr<T> {
    fn drop(&mut self) {
        unsafe {
			if !self.is_null() {
				let unknown = &*(self.ptr as *mut IUnknown);
				unknown.Release();
			}
        }
    }
}

macro_rules! safe_com_call {
	($method:ident($($parm:tt)*)) => {{
		let hr = unsafe { $method($($parm)*) };
		match hr {
			0 => Ok(()),
			winapi::shared::winerror::E_ACCESSDENIED => Err(crate::platform_layer::StartupError::AccessDenied),
			_ => Err(crate::platform_layer::StartupError::Other(format!("{}: {}", stringify!($method), hr))),
		}
	}};
	($object:ident.$method:ident($($parm:tt)*)) => {{
		let hr = unsafe { $object.$method($($parm)*) };
		match hr {
			0 => Ok(()),
			winapi::shared::winerror::E_ACCESSDENIED => Err(crate::platform_layer::StartupError::AccessDenied),
			_ => Err(crate::platform_layer::StartupError::Other(format!("{}: {}", stringify!($method), hr))),
		}
	}};
}

pub(super) fn get_run_at_startup(task: &str) -> StartupResult<bool> {
    let mut p_service: ComPtr<ITaskService> = ComPtr::default();
	safe_com_call!(CoCreateInstance(&TaskScheduler::uuidof(), 
									std::ptr::null_mut(), 
									CLSCTX_INPROC_SERVER, 
									&ITaskService::uuidof(), 
									p_service.as_mut() as *mut *mut std::os::raw::c_void))?;

	// Connect to the task service.
	safe_com_call!(p_service.Connect(Default::default(), Default::default(), Default::default(), Default::default()))?;

	let mut p_folder: ComPtr<ITaskFolder> = ComPtr::default();
	safe_com_call!(p_service.GetFolder(*ComString::new("\\"), p_folder.as_mut()))?;

	let mut p_task: ComPtr<IRegisteredTask> = ComPtr::default();
	unsafe { p_folder.GetTask(*ComString::new(task), p_task.as_mut()); }
	
	Ok(!p_task.is_null())
}

pub(super) fn set_run_at_startup(task: &str, value: bool) -> StartupResult<()> {
	unsafe { winapi::um::combaseapi::CoInitializeEx(std::ptr::null_mut(), winapi::um::objbase::COINIT_MULTITHREADED) };

	let working_path = std::fs::canonicalize(".").unwrap();
	let path = if cfg!(debug_assertions) {
		std::fs::canonicalize("../target/debug/Yaffe-rs.exe").unwrap()
	} else {
		std::fs::canonicalize("./Yaffe-rs.exe").unwrap()
	};

    let mut p_service: ComPtr<ITaskService> = ComPtr::default();
	safe_com_call!(CoCreateInstance(&TaskScheduler::uuidof(), 
									std::ptr::null_mut(), 
									CLSCTX_INPROC_SERVER, 
									&ITaskService::uuidof(), 
									p_service.as_mut() as *mut *mut std::os::raw::c_void))?;

	// Connect to the task service.
	safe_com_call!(p_service.Connect(Default::default(), Default::default(), Default::default(), Default::default()))?;

	let mut p_folder: ComPtr<ITaskFolder> = ComPtr::default();
	safe_com_call!(p_service.GetFolder(*ComString::new("\\"), p_folder.as_mut()))?;

	let mut p_task: ComPtr<IRegisteredTask> = ComPtr::default();
	unsafe { p_folder.GetTask(*ComString::new(task), p_task.as_mut()); }

	if !value {
		// If the same task exists, remove it.
		safe_com_call!(p_folder.DeleteTask(*ComString::new(task), 0))?;
	}
	else if p_task.is_null() {
		//  Create the task builder object to create the task.
		let mut p_task_definition: ComPtr<ITaskDefinition> = ComPtr::default();
		safe_com_call!(p_service.NewTask(0, p_task_definition.as_mut()))?;

		let mut p_principal: ComPtr<IPrincipal> = ComPtr::default();
		safe_com_call!(p_task_definition.get_Principal(p_principal.as_mut()))?;
		safe_com_call!(p_principal.put_RunLevel(TASK_RUNLEVEL_HIGHEST))?;

		// Create the settings for the task
		let mut p_settings: ComPtr<ITaskSettings> = ComPtr::default();
		safe_com_call!(p_task_definition.get_Settings(p_settings.as_mut()))?;

		// Set setting values for the task. 
		safe_com_call!(p_settings.put_StartWhenAvailable(VARIANT_TRUE))?;

		// Add the logon trigger to the task.
		let mut p_trig_collection: ComPtr<ITriggerCollection> = ComPtr::default();
		safe_com_call!(p_task_definition.get_Triggers(p_trig_collection.as_mut()))?;

		let mut p_trigger: ComPtr<ITrigger> = ComPtr::default();
		safe_com_call!(p_trig_collection.Create(TASK_TRIGGER_LOGON, p_trigger.as_mut()))?;

		// Add an Action to the task.
		let mut p_action_collection: ComPtr<IActionCollection> = ComPtr::default();
		safe_com_call!(p_task_definition.get_Actions(p_action_collection.as_mut()))?;

		let mut p_action: ComPtr<IAction> = ComPtr::default();
		safe_com_call!(p_action_collection.Create(TASK_ACTION_EXEC, p_action.as_mut()))?;

		let mut p_exec_action: ComPtr<IExecAction> = ComPtr::default();
		safe_com_call!(p_action.QueryInterface(&IExecAction::uuidof(), p_exec_action.as_mut() as *mut *mut std::os::raw::c_void))?;

		safe_com_call!(p_exec_action.put_Path(*ComString::new(&path.as_os_str().to_string_lossy())))?;
		safe_com_call!(p_exec_action.put_WorkingDirectory(*ComString::new(&working_path.as_os_str().to_string_lossy())))?;

		#[allow(unused_assignments)]
		#[allow(unused_variables)]
		fn bstr_variant(string: &str) -> winapi::um::oaidl::VARIANT {
			unsafe {
				let mut var: winapi::um::oaidl::VARIANT = std::mem::zeroed();
				let mut n2 = var.n1.n2_mut();
				n2.vt = winapi::shared::wtypes::VT_BSTR.try_into().unwrap();
				let n3 = n2.n3.bstrVal_mut();
				
				use std::iter::once;
				let wide: Vec<u16> = std::ffi::OsStr::new(string).encode_wide().chain(once(0)).collect();
				let root = winapi::um::oleauto::SysAllocString(wide.as_ptr());
				*n3 = root;
				var
			}
		}

		//There is also a memory leak when this call fails
		let mut group = bstr_variant("Builtin\\Administrators");
		let mut s = bstr_variant("");
		safe_com_call!(p_folder.RegisterTaskDefinition(*ComString::new(task), 
														p_task_definition.deref(), 
														TASK_CREATE_OR_UPDATE.try_into().unwrap(),
														group,
														Default::default(),
														TASK_LOGON_GROUP,
														s,
														p_task.as_mut()))?;
		unsafe { winapi::um::oleauto::SysFreeString(*group.n1.n2_mut().n3.bstrVal()); }
		unsafe { winapi::um::oleauto::SysFreeString(*s.n1.n2_mut().n3.bstrVal()); }
	}
	unsafe { winapi::um::combaseapi::CoUninitialize() };

	Ok(())
}

pub(super) fn update() -> std::io::Result<std::process::Child> {
	if std::path::Path::new("./yaffe-updater.exe").exists() {
		return std::process::Command::new("./yaffe-updater.exe").arg("./yaffe-rs.exe").spawn();
	}
	Err(std::io::Error::from(std::io::ErrorKind::NotFound))
}

pub(super) fn shutdown() -> ShutdownResult {
	use std::iter::once;
    unsafe {
        let mut token: HANDLE = std::ptr::null_mut();
        let mut tkp: TOKEN_PRIVILEGES = std::mem::zeroed();
        if OpenProcessToken(GetCurrentProcess(), TOKEN_ADJUST_PRIVILEGES | TOKEN_QUERY, &mut token) == 0 {
            return Err(std::io::Error::last_os_error());
        }

        let security_name: Vec<u16> = std::ffi::OsStr::new(SE_SHUTDOWN_NAME)
            .encode_wide()
            .chain(once(0))
            .collect();

        if LookupPrivilegeValueW(std::ptr::null(), security_name.as_ptr() as LPWSTR, &mut tkp.Privileges[0].Luid) == 0 {
            return Err(std::io::Error::last_os_error());
        }

        tkp.PrivilegeCount = 1;
        tkp.Privileges[0].Attributes = SE_PRIVILEGE_ENABLED;
        if AdjustTokenPrivileges(token, FALSE, &mut tkp, 0, std::ptr::null_mut(), std::ptr::null_mut()) == 0 {
            return Err(std::io::Error::last_os_error());
        }

		if ExitWindowsEx(EWX_SHUTDOWN | EWX_FORCEIFHUNG, SHTDN_REASON_FLAG_PLANNED) == 0 {
            return Err(std::io::Error::last_os_error());
        }
    }

    Ok(())
}

pub fn initialize_gamepad() -> Result<impl crate::input::PlatformGamepad, i32> {
    for lib_name in ["xinput1_4.dll", "xinput1_3.dll", "xinput1_2.dll", "xinput1_1.dll"].iter() {
        let handle = load(lib_name);
        if let Some(h) = handle { 
			return Ok(h); 
		}
    }
	Err(-1)
}

fn load<S: AsRef<str>>(s: S) -> Option<WindowsInput> {
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
        if !get_state_ptr.is_null() {
            opt_xinput_get_state = Some(::std::mem::transmute(get_state_ptr));
        }
    }

    unsafe { FreeLibrary(xinput_handle); }

    Some(WindowsInput { get_state: opt_xinput_get_state.unwrap(), 
                  current_state: XInputGamepadEx::default(), 
                  previous_state: XInputGamepadEx::default(),
                  last_stick_time: Instant::now(), 
                  last_button_time: Instant::now() })
}

const CONTROLLER_GUIDE: u16 = 0x0400;
type XInputGetStateFunc = unsafe extern "system" fn(DWORD, *mut XInputGamepadEx) -> DWORD;

#[repr(C)] #[derive(Default, Copy, Clone)]
struct XInputGamepadEx {
	event_count: DWORD,
	w_buttons: WORD,
	b_left_trigger: BYTE,
	b_right_trigger: BYTE,
	s_thumb_lx: SHORT,
	s_thumb_ly: SHORT,
	s_thumb_rx: SHORT,
	s_thumb_ry: SHORT,
}

struct WindowsInput {
    get_state: XInputGetStateFunc,
    previous_state: XInputGamepadEx,
    current_state: XInputGamepadEx,
    last_stick_time: Instant,
    last_button_time: Instant,
}

impl crate::input::PlatformGamepad for WindowsInput {
    fn update(&mut self, user_index: u32) -> Result<(), u32> {
		self.previous_state = self.current_state;
		
        if user_index < 4 {
            let mut output: XInputGamepadEx = unsafe { ::std::mem::zeroed() };
            let return_status = unsafe { (self.get_state)(user_index as DWORD, &mut output) };
            match return_status {
				ERROR_SUCCESS => self.current_state = output,
				ERROR_DEVICE_NOT_CONNECTED => {},
				s => return Err(s),
            } 
        }

		Ok(())
    }

    fn get_gamepad(&mut self) -> Vec<super::ControllerInput> {
		fn is_pressed(input: &WindowsInput, button: u16) -> bool {
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
		if (x * x) + (y * y) > XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE as i32 * XINPUT_GAMEPAD_LEFT_THUMB_DEADZONE as i32 &&
		   (now - self.last_stick_time).as_millis() > 100 {
			let count = result.len();
			if x < 0 && i32::abs(x) > i32::abs(y) { result.push(super::ControllerInput::DirectionLeft); }
			if y > 0 && i32::abs(y) > i32::abs(x) { result.push(super::ControllerInput::DirectionUp); }
			if y < 0 && i32::abs(y) > i32::abs(x) { result.push(super::ControllerInput::DirectionDown); }
			if x > 0 && i32::abs(x) > i32::abs(y) { result.push(super::ControllerInput::DirectionRight); }
			if result.len() > count { self.last_stick_time = now; }
		}

        result
    }
}

pub(super) fn get_clipboard(_: &glutin::window::Window) -> Option<String> {
	unsafe {
		let mut result = None;
		if OpenClipboard(std::ptr::null_mut()) != 0 {
			
			let data = GetClipboardData(CF_TEXT);
			if !data.is_null() {
				let text = GlobalLock(data);
				if !text.is_null() { 
					result = match std::ffi::CString::from_raw(text as *mut i8).into_string() {
						Err(_) => None,
						Ok(result) => Some(result),
					};
				}
					
				GlobalUnlock(data);
			}
			CloseClipboard();
		}
		result
	}
}

pub(super) fn get_and_update_volume(delta: f32) -> VolumeResult<f32> {
	unsafe { winapi::um::combaseapi::CoInitializeEx(std::ptr::null_mut(), winapi::um::objbase::COINIT_MULTITHREADED) };

    let mut p_device: ComPtr<IMMDeviceEnumerator> = ComPtr::default();
	safe_com_call!(CoCreateInstance(&MMDeviceEnumerator::uuidof(), 
									std::ptr::null_mut(), 
									CLSCTX_INPROC_SERVER, 
									&IMMDeviceEnumerator::uuidof(), 
									p_device.as_mut() as *mut *mut std::os::raw::c_void))?;

	let mut p_default: ComPtr<IMMDevice> = ComPtr::default();
	safe_com_call!(p_device.GetDefaultAudioEndpoint(eRender, eConsole, p_default.as_mut()))?;
									

	let mut p_endpoint: ComPtr<IAudioEndpointVolume> = ComPtr::default();
	safe_com_call!(p_default.Activate(&IAudioEndpointVolume::uuidof(), 
									  CLSCTX_INPROC_SERVER, 
									  std::ptr::null_mut(), 
									  p_endpoint.as_mut() as *mut *mut std::os::raw::c_void))?;

	let mut volume = 0f32;
	safe_com_call!(p_endpoint.GetMasterVolumeLevelScalar(&mut volume as *mut f32))?;

	if delta != 0. {
		volume = (volume + delta).clamp(0., 1.);
		safe_com_call!(p_endpoint.SetMasterVolumeLevelScalar(volume, std::ptr::null_mut()))?;
	}
	unsafe { winapi::um::combaseapi::CoUninitialize() };

	Ok(volume)
}

pub fn sanitize_file(file: &str) -> String {
	file.replace(['\"', '*', '<', '>', '?', '\\', '/', ':'], "")
}