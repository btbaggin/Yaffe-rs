use glutin::event::VirtualKeyCode;

#[cfg(target_os = "windows")]
#[path = "windows.rs"]
mod os;

#[cfg(any(target_os = "linux", target_os = "macos"))]
#[path = "linux.rs"]
mod os;

#[derive(std::hash::Hash, Eq, PartialEq, Copy, Clone)]
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

type StartupResult<T> = Result<T, (&'static str, i32)>;
type ShutdownResult = std::io::Result<()>;

pub fn shutdown() -> ShutdownResult {
    os::shutdown()
}

pub fn set_run_at_startup(task: &str, value: bool) -> StartupResult<()> {
    os::set_run_at_startup(task, value)
}

pub fn get_run_at_startup(task: &str) -> StartupResult<bool> {
    os::get_run_at_startup(task)
}

pub fn initialize_input() {
    os::initialize_input();
}

pub fn get_input() -> (Vec<VirtualKeyCode>, Vec<ControllerInput>) {
    //TODO need to get keyboard
    os::get_input()
}