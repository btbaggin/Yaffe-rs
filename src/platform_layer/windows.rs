use winapi::um::taskschd::*;
use winapi::um::combaseapi::*;
use winapi::shared::rpcdce::{RPC_C_AUTHN_LEVEL_PKT_PRIVACY, RPC_C_IMP_LEVEL_IMPERSONATE};
use winapi::shared::wtypesbase::CLSCTX_INPROC_SERVER;
use winapi::shared::minwindef::{WORD, BYTE, DWORD, FALSE, HKL};
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
    ExitWindowsEx, EWX_FORCEIFHUNG, EWX_SHUTDOWN, GetKeyboardState,
};
use winapi::um::winuser;
use winapi::um::libloaderapi::{FreeLibrary, GetProcAddress, LoadLibraryW};

use std::time::Instant;
use std::os::windows::ffi::OsStrExt;
use std::ops::Deref;
use std::convert::TryInto;
use glutin::event::VirtualKeyCode;
use super::{StartupResult, ShutdownResult};

struct ComString { string: BSTR }
impl ComString {
	fn new(string: &str) -> ComString {
    	use std::iter::once;
		let wide: Vec<u16> = std::ffi::OsStr::new(string).encode_wide().chain(once(0)).collect();
		let root = unsafe { winapi::um::oleauto::SysAllocString(wide.as_ptr()) };
		ComString { string: root }
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
			_ => Err((stringify!($method), hr)),
		}
	}};
	($object:ident.$method:ident($($parm:tt)*)) => {{
		let hr = unsafe { $object.$method($($parm)*) };
		match hr {
			0 => Ok(()),
			_ => Err((stringify!($method), hr)),
		}
	}};
}

pub(super) fn get_run_at_startup(task: &str) -> StartupResult<bool> {
	safe_com_call!(CoInitializeSecurity(std::ptr::null_mut(), 
										-1, 
										std::ptr::null_mut(), 
										std::ptr::null_mut(), 
										RPC_C_AUTHN_LEVEL_PKT_PRIVACY, 
										RPC_C_IMP_LEVEL_IMPERSONATE, 
										std::ptr::null_mut(), 
										0, 
										std::ptr::null_mut()))?;

    let mut p_service: ComPtr<ITaskService> = ComPtr::default();
	safe_com_call!(CoCreateInstance(&TaskScheduler::uuidof(), 
									std::ptr::null_mut(), 
									CLSCTX_INPROC_SERVER, 
									&ITaskService::uuidof(), 
									p_service.as_mut() as *mut *mut std::os::raw::c_void))?;

	// //  Connect to the task service.
	safe_com_call!(p_service.Connect(Default::default(), Default::default(), Default::default(), Default::default()))?;

	let mut p_folder: ComPtr<ITaskFolder> = ComPtr::default();
	safe_com_call!(p_service.GetFolder(*ComString::new("\\"), p_folder.as_mut()))?;

	let mut p_task: ComPtr<IRegisteredTask> = ComPtr::default();
	unsafe { p_folder.GetTask(*ComString::new(task), p_task.as_mut()); }
	
	Ok(!p_task.is_null())
}

pub(super) fn set_run_at_startup(task: &str, value: bool) -> StartupResult<()> {
	safe_com_call!(CoInitializeSecurity(std::ptr::null_mut(), 
										-1, 
										std::ptr::null_mut(), 
										std::ptr::null_mut(), 
										RPC_C_AUTHN_LEVEL_PKT_PRIVACY, 
										RPC_C_IMP_LEVEL_IMPERSONATE, 
										std::ptr::null_mut(), 
										0, 
										std::ptr::null_mut()))?;

	let working_path = std::fs::canonicalize(".").unwrap();
	let path = if cfg!(debug_assertions) {
		std::fs::canonicalize("./target/debug/Yaffe-rs.exe").unwrap()
	} else {
		std::fs::canonicalize(".Yaffe-rs.exe").unwrap()
	};

    let mut p_service: ComPtr<ITaskService> = ComPtr::default();
	safe_com_call!(CoCreateInstance(&TaskScheduler::uuidof(), 
									std::ptr::null_mut(), 
									CLSCTX_INPROC_SERVER, 
									&ITaskService::uuidof(), 
									p_service.as_mut() as *mut *mut std::os::raw::c_void))?;

	// //  Connect to the task service.
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

		// //  Create the settings for the task
		let mut p_settings: ComPtr<ITaskSettings> = ComPtr::default();
		safe_com_call!(p_task_definition.get_Settings(p_settings.as_mut()))?;

		// //  Set setting values for the task. 
		safe_com_call!(p_settings.put_StartWhenAvailable(VARIANT_TRUE))?;

		// //  Add the logon trigger to the task.
		let mut p_trig_collection: ComPtr<ITriggerCollection> = ComPtr::default();
		safe_com_call!(p_task_definition.get_Triggers(p_trig_collection.as_mut()))?;

		let mut p_trigger: ComPtr<ITrigger> = ComPtr::default();
		safe_com_call!(p_trig_collection.Create(TASK_TRIGGER_LOGON, p_trigger.as_mut()))?;

		// //  Add an Action to the task.
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
				let mut n3 = n2.n3.bstrVal_mut();
				
				use std::iter::once;
				let wide: Vec<u16> = std::ffi::OsStr::new(string).encode_wide().chain(once(0)).collect();
				let mut root = winapi::um::oleauto::SysAllocString(wide.as_ptr());
				n3 = &mut root;
				var
			}
		}

		//TODO this method call doesnt work
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

	Ok(())
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

pub fn initialize_input() -> Result<impl crate::input::PlatformInput, i32> {
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

	let hkl = unsafe { winuser::GetKeyboardLayout(0) };
    Some(WindowsInput { get_state: opt_xinput_get_state.unwrap(), 
                  current_state: (XInputGamepadEx::default(), [0; 256]), 
                  previous_state: (XInputGamepadEx::default(), [0; 256]),
                  last_stick_time: Instant::now(), 
                  last_button_time: Instant::now(),
				  hkl: hkl })
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
    previous_state: (XInputGamepadEx, [u8; 256]),
    current_state: (XInputGamepadEx, [u8; 256]),
    last_stick_time: Instant,
    last_button_time: Instant,
	hkl: HKL
}

impl crate::input::PlatformInput for WindowsInput {
    fn update(&mut self, user_index: u32) -> Result<(), u32> {
		self.previous_state = self.current_state;
        if user_index < 4 {
            let mut output: XInputGamepadEx = unsafe { ::std::mem::zeroed() };
            let return_status = unsafe { (self.get_state)(user_index as DWORD, &mut output) };
            match return_status {
				ERROR_SUCCESS => self.current_state.0 = output,
				ERROR_DEVICE_NOT_CONNECTED => {},
				s => return Err(s),
            } 
        }

		let mut output: [u8; 256] = [0; 256];
		let result = unsafe { GetKeyboardState(output.as_mut_ptr()) };
		if result != 0 { self.current_state.1 = output; }
		else { return Err(result as u32); }

		return Ok(());
    }

    fn get_gamepad(&mut self) -> Vec<super::ControllerInput> {
		fn is_pressed(input: &WindowsInput, button: u16) -> bool {
			input.current_state.0.w_buttons & button != 0 && input.previous_state.0.w_buttons & button == 0
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

		let x = self.current_state.0.s_thumb_lx as i32;
		let y = self.current_state.0.s_thumb_ly as i32;
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

	fn get_keyboard(&mut self) -> Vec<(VirtualKeyCode, Option<char>)> {
		let mut result = Vec::new();
		let now = Instant::now();

		for i in 0..255 {
			if (self.current_state.1[i] & 0x80) != 0 &&
			   (self.previous_state.1[i] & 0x80) == 0 {
				   self.last_button_time = now;

				   let key = match i as i32 {
					winuser::VK_BACK => Some(VirtualKeyCode::Back),
					winuser::VK_TAB => Some(VirtualKeyCode::Tab),
					winuser::VK_RETURN => Some(VirtualKeyCode::Return),

					winuser::VK_PAUSE => Some(VirtualKeyCode::Pause),
					winuser::VK_ESCAPE => Some(VirtualKeyCode::Escape),
					winuser::VK_SPACE => Some(VirtualKeyCode::Space),
					winuser::VK_PRIOR => Some(VirtualKeyCode::PageUp),
					winuser::VK_NEXT => Some(VirtualKeyCode::PageDown),
					winuser::VK_END => Some(VirtualKeyCode::End),
					winuser::VK_HOME => Some(VirtualKeyCode::Home),
					winuser::VK_LEFT => Some(VirtualKeyCode::Left),
					winuser::VK_UP => Some(VirtualKeyCode::Up),
					winuser::VK_RIGHT => Some(VirtualKeyCode::Right),
					winuser::VK_DOWN => Some(VirtualKeyCode::Down),
					winuser::VK_INSERT => Some(VirtualKeyCode::Insert),
					winuser::VK_DELETE => Some(VirtualKeyCode::Delete),
					0x30 => Some(VirtualKeyCode::Key0),
					0x31 => Some(VirtualKeyCode::Key1),
					0x32 => Some(VirtualKeyCode::Key2),
					0x33 => Some(VirtualKeyCode::Key3),
					0x34 => Some(VirtualKeyCode::Key4),
					0x35 => Some(VirtualKeyCode::Key5),
					0x36 => Some(VirtualKeyCode::Key6),
					0x37 => Some(VirtualKeyCode::Key7),
					0x38 => Some(VirtualKeyCode::Key8),
					0x39 => Some(VirtualKeyCode::Key9),
					0x41 => Some(VirtualKeyCode::A),
					0x42 => Some(VirtualKeyCode::B),
					0x43 => Some(VirtualKeyCode::C),
					0x44 => Some(VirtualKeyCode::D),
					0x45 => Some(VirtualKeyCode::E),
					0x46 => Some(VirtualKeyCode::F),
					0x47 => Some(VirtualKeyCode::G),
					0x48 => Some(VirtualKeyCode::H),
					0x49 => Some(VirtualKeyCode::I),
					0x4A => Some(VirtualKeyCode::J),
					0x4B => Some(VirtualKeyCode::K),
					0x4C => Some(VirtualKeyCode::L),
					0x4D => Some(VirtualKeyCode::M),
					0x4E => Some(VirtualKeyCode::N),
					0x4F => Some(VirtualKeyCode::O),
					0x50 => Some(VirtualKeyCode::P),
					0x51 => Some(VirtualKeyCode::Q),
					0x52 => Some(VirtualKeyCode::R),
					0x53 => Some(VirtualKeyCode::S),
					0x54 => Some(VirtualKeyCode::T),
					0x55 => Some(VirtualKeyCode::U),
					0x56 => Some(VirtualKeyCode::V),
					0x57 => Some(VirtualKeyCode::W),
					0x58 => Some(VirtualKeyCode::X),
					0x59 => Some(VirtualKeyCode::Y),
					0x5A => Some(VirtualKeyCode::Z),
					winuser::VK_NUMPAD0 => Some(VirtualKeyCode::Numpad0),
					winuser::VK_NUMPAD1 => Some(VirtualKeyCode::Numpad1),
					winuser::VK_NUMPAD2 => Some(VirtualKeyCode::Numpad2),
					winuser::VK_NUMPAD3 => Some(VirtualKeyCode::Numpad3),
					winuser::VK_NUMPAD4 => Some(VirtualKeyCode::Numpad4),
					winuser::VK_NUMPAD5 => Some(VirtualKeyCode::Numpad5),
					winuser::VK_NUMPAD6 => Some(VirtualKeyCode::Numpad6),
					winuser::VK_NUMPAD7 => Some(VirtualKeyCode::Numpad7),
					winuser::VK_NUMPAD8 => Some(VirtualKeyCode::Numpad8),
					winuser::VK_NUMPAD9 => Some(VirtualKeyCode::Numpad9),
					winuser::VK_MULTIPLY => Some(VirtualKeyCode::NumpadMultiply),
					winuser::VK_ADD => Some(VirtualKeyCode::NumpadAdd),
					winuser::VK_SUBTRACT => Some(VirtualKeyCode::NumpadSubtract),
					winuser::VK_DECIMAL => Some(VirtualKeyCode::NumpadDecimal),
					winuser::VK_DIVIDE => Some(VirtualKeyCode::NumpadDivide),
					winuser::VK_F1 => Some(VirtualKeyCode::F1),
					winuser::VK_F2 => Some(VirtualKeyCode::F2),
					winuser::VK_F3 => Some(VirtualKeyCode::F3),
					winuser::VK_F4 => Some(VirtualKeyCode::F4),
					winuser::VK_F5 => Some(VirtualKeyCode::F5),
					winuser::VK_F6 => Some(VirtualKeyCode::F6),
					winuser::VK_F7 => Some(VirtualKeyCode::F7),
					winuser::VK_F8 => Some(VirtualKeyCode::F8),
					winuser::VK_F9 => Some(VirtualKeyCode::F9),
					winuser::VK_F10 => Some(VirtualKeyCode::F10),
					winuser::VK_F11 => Some(VirtualKeyCode::F11),
					winuser::VK_F12 => Some(VirtualKeyCode::F12),
					winuser::VK_NUMLOCK => Some(VirtualKeyCode::Numlock),
					winuser::VK_OEM_PLUS => Some(VirtualKeyCode::Equals),
					winuser::VK_OEM_COMMA => Some(VirtualKeyCode::Comma),
					winuser::VK_OEM_MINUS => Some(VirtualKeyCode::Minus),
					winuser::VK_OEM_PERIOD => Some(VirtualKeyCode::Period),
					winuser::VK_OEM_1 => Some(VirtualKeyCode::Colon),
					winuser::VK_OEM_2 => Some(VirtualKeyCode::Slash),
					winuser::VK_OEM_3 => Some(VirtualKeyCode::Grave),
					winuser::VK_OEM_4 => Some(VirtualKeyCode::LBracket),
					winuser::VK_OEM_5 => Some(VirtualKeyCode::Backslash),
					winuser::VK_OEM_6 => Some(VirtualKeyCode::RBracket),
					winuser::VK_OEM_7 => Some(VirtualKeyCode::Apostrophe),
					_ => None,
					};
					if let Some(k) = key {
						let c = unsafe { get_char(&self.current_state.1, i as u32, self.hkl) };
						result.push((k, c));
					}
			    }
		}

		result
	}

    fn get_modifiers(&mut self) -> glutin::event::ModifiersState {
		use glutin::event::ModifiersState;
		fn is_down(input: &WindowsInput, key: i32) -> bool {
			(input.current_state.1[key as usize] & 0x80) != 0 
		}
		let mut result = ModifiersState::empty();

		if is_down(self, winuser::VK_LSHIFT) || is_down(self, winuser::VK_RSHIFT) { result.insert(ModifiersState::SHIFT) }
		if is_down(self, winuser::VK_LCONTROL) || is_down(self, winuser::VK_RCONTROL) { result.insert(ModifiersState::CTRL) }
		if is_down(self, winuser::VK_LWIN) || is_down(self, winuser::VK_RWIN) { result.insert(ModifiersState::LOGO) }
		if is_down(self, winuser::VK_LMENU) || is_down(self, winuser::VK_RMENU) { result.insert(ModifiersState::ALT)}

		result
	}
}

unsafe fn get_char(keyboard_state: &[u8; 256], v_key: u32, hkl: HKL) -> Option<char> {
    let mut unicode_bytes = [0u16; 5];
    let len = winuser::ToUnicodeEx(
        v_key,
        0,
        keyboard_state.as_ptr(),
        unicode_bytes.as_mut_ptr(),
        unicode_bytes.len() as _,
        0,
        hkl,
    );
    if len >= 1 {
        std::char::decode_utf16(unicode_bytes.iter().cloned())
            .next()
            .and_then(|c| c.ok())
    } else {
        None
    }
}

pub(super) fn get_clipboard() -> Option<String> {
	unsafe {
		let mut result = None;
		if winuser::OpenClipboard(std::ptr::null_mut()) != 0 {
			
			let data = winuser::GetClipboardData(winuser::CF_TEXT);
			if data.is_null() { return None; }
			
			let text = GlobalLock(data);
			if !text.is_null() { 
				result = match std::ffi::CString::from_raw(text as *mut i8).into_string() {
					Err(_) => None,
					Ok(result) => Some(result),
				};
			}
				
			GlobalUnlock(data);
			winuser::CloseClipboard();
		}
		result
	}
}