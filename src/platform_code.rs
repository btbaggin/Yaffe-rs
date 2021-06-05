use winapi::um::taskschd::*;
use winapi::um::combaseapi::*;
use winapi::shared::rpcdce::{RPC_C_AUTHN_LEVEL_PKT_PRIVACY, RPC_C_IMP_LEVEL_IMPERSONATE};
use winapi::shared::wtypesbase::CLSCTX_INPROC_SERVER;
use winapi::{Interface, Class};
use winapi::um::unknwnbase::IUnknown;
use winapi::shared::winerror::HRESULT;
use winapi::shared::wtypes::{BSTR, VARIANT_TRUE};
use std::os::windows::ffi::OsStrExt;
use std::ops::Deref;
use std::convert::TryInto;

type ComResult<T> = Result<T, (&'static str, HRESULT)>;

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

#[cfg(windows)]
pub fn get_run_at_startup(task: &str) -> ComResult<bool> {
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

#[cfg(windows)]
pub fn set_run_at_startup(task: &str, value: bool) -> ComResult<()> {
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