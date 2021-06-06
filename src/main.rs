#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::any::Any;
use std::time::Instant;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use druid_shell::kurbo::Size;
use druid_shell::piet::Piet;
use druid_shell::{
    Application, KeyEvent, Code,
    Region, WinHandler, WindowBuilder, WindowHandle,
};

use crate::logger::{UserMessage, LogEntry};

/*
    TODO:
    tranparent window
    hide overlay window
    button remapping?
*/

pub mod colors {
    use druid_shell::piet::{Color};
    pub const MENU_BACKGROUND: Color = Color::rgba8(64, 64, 64, 128);
    pub const MODAL_OVERLAY_COLOR: Color = Color::rgba8(0, 0, 0, 115);
    pub const MODAL_BACKGROUND: Color = Color::rgba8(26, 26, 26, 255);
    
    const TEXT_FOCUSED: Color = Color::rgba8(242, 242, 242, 255);
    pub fn get_font_color(settings: &crate::settings::SettingsFile) -> Color {
        let (r, g, b, a) = settings.get_color("font_color", &TEXT_FOCUSED).as_rgba();
        Color::rgba(r, g, b, a)
    }
    pub fn get_font_unfocused_color(settings: &crate::settings::SettingsFile) -> Color {
        let color = settings.get_color("font_color", &TEXT_FOCUSED);
        change_brightness(color, -0.4)
    }
    
    const ACCENT_COLOR: Color = Color::rgba8(64, 77, 255, 255);
    pub fn get_accent_color(settings: &crate::settings::SettingsFile) -> &Color {
        settings.get_color("accent_color", &ACCENT_COLOR)
    }
    pub fn get_accent_unfocused_color(settings: &crate::settings::SettingsFile) -> Color {
        let color = settings.get_color("accent_color", &ACCENT_COLOR);
        change_brightness(color, -0.3)
    }

    pub fn change_brightness(color: &Color, factor: f64) -> Color {
        let (mut r, mut g, mut b, a) = color.as_rgba();

        if factor < 0. {
            let factor = 1. + factor;
            r *= factor;
            g *= factor;
            b *= factor;
        } else {
            r = (1. - r) * factor + r;
            g = (1. - g) * factor + g;
            b  = (1. - b) * factor + b;
        }

        return Color::rgba(r, g, b, a);
    }
}

pub mod font {
    const INFO_FONT_SIZE: f64 = 18.;
    const TITLE_SIZE: f64 = 32.;
    pub const FONT_SIZE: f64 = 24.;
    pub fn get_info_font_size(state: &crate::YaffeState) -> f64 {
        *state.settings.get_f64("info_font_size", &INFO_FONT_SIZE)
    }
    pub fn get_title_font_size(state: &crate::YaffeState) -> f64 {
        *state.settings.get_f64("title_font_size", &TITLE_SIZE)
    }
}

pub mod ui {
    pub const MARGIN: f64 = 10.;
    pub const LABEL_SIZE: f64 = 200.;
}

#[macro_use]
extern crate lazy_static;

mod widgets;
mod assets;
mod database;
mod modals;
mod platform;
mod platform_code;
mod overlay;
mod restrictions;
mod server;
mod job_system;
mod logger;
mod controller;
mod settings;
use widgets::*;
use overlay::OverlayWindow;
use restrictions::RestrictedMode;
pub use modals::{display_modal};
pub use job_system::{JobQueue, JobType, RawDataPointer};

#[derive(Clone, Copy)]
pub enum Actions {
    Info,
    Accept,
    Select,
    Back,
    Up,
    Down,
    Left,
    Right,
    Filter,
    KeyPress(u32),
}

#[derive(Clone, Copy)]
pub enum SystemActions {
    ShowOverlay,
    ShowMenu,
}

use std::hash::Hash;
struct InputMap<A: Eq + Hash, B: Eq + Hash, T: Clone> {
    keys: HashMap<A, T>,
    cont: HashMap<B, T>,
}
impl<A: Eq + Hash, B: Eq + Hash, T: Clone> InputMap<A, B, T> {
    fn new() -> InputMap<A, B, T> {
        InputMap {
            keys: HashMap::new(),
            cont: HashMap::new(),
        }
    }

    fn insert(&mut self, code: A, button: B, action: T) {
        self.keys.insert(code, action.clone());
        self.cont.insert(button, action);
    }

    fn get(&self, code: Option<A>, button: Option<B>) -> Option<&T> {
        if let Some(c) = code {
            return self.keys.get(&c);
        } else if let Some(b) = button {
            return self.cont.get(&b);
        }
        return None;
    }
}

lazy_static! {
    static ref ACTION_MAP: InputMap<Code, u16, Actions> = {
        let mut m = InputMap::new();
        m.insert(Code::KeyI, controller::CONTROLLER_X, Actions::Info);
        m.insert(Code::KeyF, controller::CONTROLLER_Y, Actions::Filter);
        m.insert(Code::Enter, controller::CONTROLLER_A, Actions::Accept);
        m.insert(Code::Escape, controller::CONTROLLER_B, Actions::Back);
        m.insert(Code::ArrowUp, controller::CONTROLLER_UP, Actions::Up);
        m.insert(Code::ArrowDown, controller::CONTROLLER_DOWN, Actions::Down);
        m.insert(Code::ArrowRight, controller::CONTROLLER_RIGHT, Actions::Right);
        m.insert(Code::ArrowLeft, controller::CONTROLLER_LEFT, Actions::Left);
        m.insert(Code::Tab, controller::CONTROLLER_START, Actions::Select);
        m
    };
}
lazy_static! {
    static ref SYSTEM_ACTION_MAP: InputMap<Code, u16, SystemActions> = {
        let mut m = InputMap::new();
        m.insert(Code::F1, controller::CONTROLLER_START, SystemActions::ShowMenu);
        m.insert(Code::KeyO, controller::CONTROLLER_GUIDE, SystemActions::ShowOverlay);
        m
    };
}


pub struct Platform {
    id: i64,
    name: String,
    path: String,
    apps: Vec<Executable>,
    kind: platform::PlatformType,
}

pub struct Executable {
    file: String,
    name: String,
    overview: String,
    rating: platform::Rating,
    players: u8,
    platform_id: i64, //Duplicated from Platform so we always know it, even if launching from recents
    boxart: Rc<RefCell<assets::AssetSlot>>,
    banner: Rc<RefCell<assets::AssetSlot>>,
}

#[derive(Default)]
struct YaffeWin {
    size: Size,
    handle: WindowHandle,
}

pub struct YaffeState {
    win: YaffeWin,
    last_time: Instant,
    delta_time: f64,
    overlay: *mut OverlayWindow,
    selected_platform: usize,
    selected_app: usize,
    platforms: Vec<Platform>,
    search_info: widgets::SearchInfo,
    focused_widget: widgets::WidgetId,
    restricted_mode: RestrictedMode,
    restricted_last_approve: Option<Instant>,
    modals: std::sync::Mutex<Vec<modals::Modal>>,
    queue: Arc<RefCell<job_system::JobQueue>>,
    refresh_list: bool,
    controller: Option<controller::XInput>,
    settings: settings::SettingsFile,
}
impl YaffeState {
    fn new(overlay: *mut OverlayWindow, 
           settings: settings::SettingsFile, 
           queue: Arc<RefCell<job_system::JobQueue>>) -> YaffeState {
        YaffeState {
            win: YaffeWin::default(),
            last_time: Instant::now(),
            delta_time: 0.,
            overlay: overlay,
            selected_platform: 0,
            selected_app: 0,
            platforms: vec!(),
            search_info: SearchInfo::new(),
            focused_widget: get_widget_id!(widgets::PlatformList),
            restricted_mode: RestrictedMode::Off,
            restricted_last_approve: None,
            modals: std::sync::Mutex::new(vec!()),
            queue: queue,
            refresh_list: true,
            controller: controller::load_xinput(),
            settings: settings,
        }
    }

    fn get_platform(&self) -> &Platform {
        &self.platforms[self.selected_platform]
    }

    fn get_executable(&self) -> Option<&Executable> {
        let p = &self.get_platform();
        if p.apps.len() > self.selected_app { 
            return Some(&p.apps[self.selected_app]);
        }
        None
    }

    pub fn get_overlay(&self) -> &mut OverlayWindow {
        unsafe { &mut *self.overlay }
    }

    fn is_widget_focused(&self, widget: &impl WidgetName) -> bool {
        if self.focused_widget == widget.get_id() {
            return true;
        }
        false
    }
}

impl WinHandler for WidgetTree {
    fn connect(&mut self, handle: &WindowHandle) { 
        self.data.win.handle = handle.clone(); 
        server::start_up();

        //Attempt to start COM here since it doesnt work in the settings modal?
        #[cfg(windows)]
	    unsafe { winapi::um::combaseapi::CoInitializeEx(std::ptr::null_mut(), winapi::um::objbase::COINIT_MULTITHREADED) };
}
    fn prepare_paint(&mut self) { self.data.win.handle.invalidate(); }

    fn paint(&mut self, piet: &mut Piet, _: &Region) {
        if let Err(e) = settings::update_settings(&mut self.data.settings) {
            logger::log_entry_with_message(logger::LogTypes::Warning, e, "Unable to retrieve updated settings");
        }

        if !self.data.get_overlay().process_is_running() {
            assets::load_texture_atlas(piet);

            check_controller_input(self);

            let now = Instant::now();
            self.data.delta_time = (now - self.data.last_time).as_millis() as f64 / 1000.;
            self.data.last_time = now;

            let size = self.data.win.size;
            let window_rect = size.to_rect();

            if self.data.refresh_list {
                platform::get_database_info(&mut self.data);
                self.data.refresh_list = false;
            }

            self.data.focused_widget = *self.focus.last().unwrap();
            self.render_all(window_rect, piet, !self.layout_valid);
            self.layout_valid = true;

            crate::widgets::run_animations(self, self.data.delta_time);

            let modals = self.data.modals.lock().unwrap();
            if let Some(m) = modals.last() {
                modals::modal::render_modal(&self.data.settings, m, &window_rect, piet);
            }
        }

        self.data.win.handle.request_anim_frame();
    }

    fn key_down(&mut self, event: KeyEvent) -> bool {
        if self.data.get_overlay().process_is_running() { return false; }
        handle_input(self, Some(event.code), None)
    }

    fn size(&mut self, size: Size) { 
        self.data.win.size = size; 
        self.layout_valid = false;
    }
    // fn got_focus(&mut self) { println!("Got focus"); }
    // fn lost_focus(&mut self) { println!("Lost focus"); }
    fn request_close(&mut self) { 
        self.data.win.handle.close(); 

        #[cfg(windows)]
        unsafe { winapi::um::combaseapi::CoUninitialize() };

    }
    fn destroy(&mut self) { 
        server::shutdown();
        Application::global().quit() 
    }
    fn as_any(&mut self) -> &mut dyn Any { self }
}

fn main() {
    let app = Application::new().unwrap();

    logger::initialize_log();
    let queue = job_system::start_job_system();

    let settings = match settings::load_settings("./settings.txt") {
        Ok(settings) => settings,
        Err(e) => {
            logger::log_entry(logger::LogTypes::Error, e);
            settings::SettingsFile::default()
        },
    };

    let q = Arc::new(RefCell::new(queue));
    let root = build_ui_tree(q.clone());
    let state = YaffeState::new(overlay::OverlayWindow::new(), settings, q.clone());

    assets::initialize_asset_cache();

    let mut ui = widgets::WidgetTree::new(root, state);
    ui.focus(std::any::TypeId::of::<widgets::PlatformList>());

    let mut builder = WindowBuilder::new(app.clone());
    builder.set_handler(Box::new(ui));
    builder.set_title("Yaffe");

    let window = builder.build().unwrap();
    window.show();

    app.run(None);
}

fn build_ui_tree(queue: Arc<RefCell<job_system::JobQueue>>) -> WidgetContainer {
    let mut root = WidgetContainer::root(widgets::Background::new(queue.clone()));
    root.add_child(widgets::PlatformList::new(queue.clone()),Size::new(0.25, 1.))
        .with_child(widgets::AppList::new(queue.clone()), Size::new(0.75, 1.))
            .add_child(widgets::SearchBar::new(queue.clone()), Size::new(1., 0.05))
            .add_child(widgets::Toolbar::new(queue.clone()), Size::new(1., 0.075))
            .add_child(widgets::InfoPane::new(queue.clone()), Size::new(0.33, 1.))
            .orientation(ContainerOrientation::Floating);
            
    root
}

fn on_menu_close(state: &mut YaffeState, result: modals::ModalResult, content: &Box<dyn modals::ModalContent>) {
    if let modals::ModalResult::Ok = result {
        let list_content = content.as_any().downcast_ref::<modals::ListModal<String>>().unwrap();
        
        match &list_content.get_selected()[..] {
            "Add Emulator" => {
                let content = Box::new(modals::PlatformDetailModal::emulator());
                display_modal(state, "New Emulator", Some("Confirm"), content, modals::ModalSize::Half, Some(modals::on_add_platform_close));
            },
            "Add Application" => {
                let content = Box::new(modals::PlatformDetailModal::application());
                display_modal(state, "New Application", Some("Confirm"), content, modals::ModalSize::Half, Some(modals::on_add_platform_close));
            },
            "Settings" => {
                let content = Box::new(modals::SettingsModal::new());
                display_modal(state, "Settings", Some("Confirm"), content, modals::ModalSize::Third, Some(modals::on_settings_close));
            },
            "Disable Restricted Mode" => {
                restrictions::verify_restricted_action(state, |state| { 
                    let state = state.downcast_mut::<YaffeState>().unwrap();
                    state.restricted_mode = RestrictedMode::Off; 
                });
            },
            "Enable Restricted Mode" => {
                state.restricted_mode = RestrictedMode::Pending;
                let content = Box::new(modals::SetRestrictedModal::new());
                display_modal(state, "Restricted Mode", Some("Set passcode"), content, modals::ModalSize::Third, Some(restrictions::on_restricted_modal_close))
            },
            "Exit Yaffe" => state.win.handle.close(), //This causes issues. not sure why
            "Shut Down" => { system_shutdown::shutdown().display_failure("Failed to shut down", state); },
            _ => panic!("Unknown menu option"),
        }
    }
}

fn check_controller_input(tree: &mut WidgetTree) {
    if let Some(controller) = &mut tree.data.controller {
        if controller.update(0).is_ok() {
            for e in controller.get_actions() {
                handle_input(tree, None, Some(e));
            }
        }
    }   
}

fn handle_input(tree: &mut WidgetTree, code: Option<Code>, button: Option<u16>) -> bool {
    if let Some(action) = SYSTEM_ACTION_MAP.get(code, button) { 
        match action {
            SystemActions::ShowMenu => {
                let mut l = Box::new(modals::ListModal::new(None));
                l.add_item(String::from("Exit Yaffe"));
                l.add_item(String::from("Shut Down"));
                l.add_item(String::from("Settings"));
                match tree.data.restricted_mode {
                    RestrictedMode::On(_) => l.add_item(String::from("Disable Restricted Mode")),
                    RestrictedMode::Off => l.add_item(String::from("Enable Restricted Mode")),
                    RestrictedMode::Pending => {},
                }
                l.add_item(String::from("Add Emulator"));
                l.add_item(String::from("Add Application"));
    
                display_modal(&mut tree.data, "Menu", None, l, modals::ModalSize::Third, Some(on_menu_close));
            },
            SystemActions::ShowOverlay => {
                let o = tree.data.get_overlay();
                o.show(); 
            },
        }
        return true;
    }

    let action_code = match ACTION_MAP.get(code, button) {
                    Some(a) => Some(*a),
                    None => {
                        if let Some(c) = code { Some(Actions::KeyPress(c as u32)) }
                        else if let Some(b) = button { Some(Actions::KeyPress(b.into())) }
                        else { None }
                    }
    };

    if let Some(action) = action_code {
        if !modals::modal::is_modal_open(&tree.data) {
            let mut handler = DeferredAction::new();
            let focus = tree.focus.last().log_if_fail();

            tree.root.action(&mut tree.data, &action, focus, &mut handler);
            handler.resolve(tree);
        } else {
            modals::modal::update_modal(&mut tree.data, &action);
        }
    }
    
    false
}