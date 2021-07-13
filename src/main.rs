#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::Instant;
use std::collections::HashMap;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use speedy2d::*;
use speedy2d::window::*;
use speedy2d::shape::Rectangle;
use speedy2d::dimen::Vector2;
pub use crate::settings::SettingNames;

use crate::logger::{UserMessage, LogEntry};

type V2 = Vector2<f32>;

/*
    TODO:
    tranparent window
    hide overlay window
    button remapping?
*/

pub mod colors {
    use speedy2d::color::Color;
    pub const MENU_BACKGROUND: Color = Color::from_rgba(0.25, 0.25, 0.25, 0.5);
    pub const MODAL_OVERLAY_COLOR: Color = Color::from_rgba(0., 0., 0., 0.6);
    pub const MODAL_BACKGROUND: Color = Color::from_rgba(0.1, 0.1, 0.1, 1.);
    
    pub fn get_font_color(settings: &crate::settings::SettingsFile) -> Color {
        settings.get_color(crate::SettingNames::FontColor).clone()
    }
    pub fn get_font_unfocused_color(settings: &crate::settings::SettingsFile) -> Color {
        let color = settings.get_color(crate::SettingNames::FontColor);
        change_brightness(&color, -0.4)
    }
    
    pub fn get_accent_color(settings: &crate::settings::SettingsFile) -> Color {
        settings.get_color(crate::SettingNames::AccentColor)
    }
    pub fn get_accent_unfocused_color(settings: &crate::settings::SettingsFile) -> Color {
        let color = settings.get_color(crate::SettingNames::AccentColor);
        change_brightness(&color, -0.3)
    }

    pub fn change_brightness(color: &Color, factor: f32) -> Color {
        let mut r = color.r();
        let mut g = color.g();
        let mut b = color.b();
        let a = color.a();

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

        return Color::from_rgba(r, g, b, a);
    }
}

pub mod font {
    pub const FONT_SIZE: f32 = 24.;
    pub fn get_info_font_size(state: &crate::YaffeState) -> f32 {
        state.settings.get_f32(crate::SettingNames::InfoFontSize)
    }
    pub fn get_title_font_size(state: &crate::YaffeState) -> f32 {
        state.settings.get_f32(crate::SettingNames::TitleFontSize)
    }
}

pub mod ui {
    pub const MARGIN: f32 = 10.;
    pub const LABEL_SIZE: f32 = 200.;
}

trait Rect {
    fn left(&self) -> f32;
    fn right(&self) -> f32;
    fn top(&self) -> f32;
    fn bottom(&self) -> f32;
    fn point_and_size(pos: V2, size: V2) -> Self;
}
impl Rect for speedy2d::shape::Rectangle {
    fn left(&self) -> f32 { self.top_left().x }
    fn right(&self) -> f32 { self.bottom_right().x }
    fn top(&self) -> f32 { self.top_left().y }
    fn bottom(&self) -> f32 { self.bottom_right().y }
    fn point_and_size(pos: V2, size: V2) -> Self { speedy2d::shape::Rectangle::new(pos, pos + size) }
}

trait Transparent {
    fn with_alpha(&self, alpha: f32) -> Self;
}
impl Transparent for speedy2d::color::Color {
    fn with_alpha(&self, alpha: f32) -> Self {
        speedy2d::color::Color::from_rgba(self.r(), self.g(), self.b(), alpha)
    }
}

#[macro_use]
extern crate lazy_static;

mod widgets;
mod assets;
mod database;
mod modals;
mod platform;
mod platform_layer;
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
    ToggleOverlay,
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
    static ref ACTION_MAP: InputMap<VirtualKeyCode, u16, Actions> = {
        let mut m = InputMap::new();
        m.insert(VirtualKeyCode::I, controller::CONTROLLER_X, Actions::Info);
        m.insert(VirtualKeyCode::F, controller::CONTROLLER_Y, Actions::Filter);
        m.insert(VirtualKeyCode::Return, controller::CONTROLLER_A, Actions::Accept);
        m.insert(VirtualKeyCode::Escape, controller::CONTROLLER_B, Actions::Back);
        m.insert(VirtualKeyCode::Up, controller::CONTROLLER_UP, Actions::Up);
        m.insert(VirtualKeyCode::Down, controller::CONTROLLER_DOWN, Actions::Down);
        m.insert(VirtualKeyCode::Right, controller::CONTROLLER_RIGHT, Actions::Right);
        m.insert(VirtualKeyCode::Left, controller::CONTROLLER_LEFT, Actions::Left);
        m.insert(VirtualKeyCode::Tab, controller::CONTROLLER_START, Actions::Select);
        m
    };
}
lazy_static! {
    static ref SYSTEM_ACTION_MAP: InputMap<VirtualKeyCode, u16, SystemActions> = {
        let mut m = InputMap::new();
        m.insert(VirtualKeyCode::F1, controller::CONTROLLER_START, SystemActions::ShowMenu);
        m.insert(VirtualKeyCode::O, controller::CONTROLLER_GUIDE, SystemActions::ToggleOverlay);
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

struct YaffeWin {
    size: speedy2d::dimen::Vector2<u32>,
}

pub struct YaffeState {
    win: YaffeWin,
    last_time: Instant,
    delta_time: f32,
    overlay: Rc<RefCell<OverlayWindow>>,
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
    running: bool,
}
impl YaffeState {
    fn new(overlay: Rc<RefCell<OverlayWindow>>, 
           settings: settings::SettingsFile, 
           queue: Arc<RefCell<job_system::JobQueue>>) -> YaffeState {
        YaffeState {
            win: YaffeWin { size: speedy2d::dimen::Vector2::new(0, 0) },
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
            running: true,
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

    fn is_widget_focused(&self, widget: &impl WidgetName) -> bool {
        if self.focused_widget == widget.get_id() {
            return true;
        }
        false
    }

    fn overlay_is_active(&self) -> bool {
        let mut overlay = self.overlay.borrow_mut();
        overlay.process_is_running()
    }
}

impl WindowHandler  for WidgetTree {
    fn on_start(&mut self, _: &mut WindowHelper, info: WindowStartupInfo) {
        self.data.win.size = *info.viewport_size_pixels();
        server::start_up();
    
        //Attempt to start COM here since it doesnt work in the settings modal?
        #[cfg(windows)]
	    unsafe { winapi::um::combaseapi::CoInitializeEx(std::ptr::null_mut(), winapi::um::objbase::COINIT_MULTITHREADED) };
    }

    fn on_draw(&mut self, helper: &mut WindowHelper, graphics: &mut Graphics2D) {
        if let Err(e) = settings::update_settings(&mut self.data.settings) {
            logger::log_entry_with_message(logger::LogTypes::Warning, e, "Unable to retrieve updated settings");
        }

        if !self.data.running {
            shutdown(helper);
            return;
        }

        assets::load_texture_atlas(graphics);

        if !self.data.overlay_is_active() {
            if let Some(controller) = &mut self.data.controller {
                for e in controller.get_actions(0) {
                    handle_input(self, None, Some(e));
                }
            }  

            let now = Instant::now();
            self.data.delta_time = (now - self.data.last_time).as_millis() as f32 / 1000.;
            self.data.last_time = now;

            let size = self.data.win.size;
            let window_rect = Rectangle::from_tuples((0., 0.), (size.x as f32, size.y as f32));

            if self.data.refresh_list {
                platform::get_database_info(&mut self.data);
                self.data.refresh_list = false;
            }

            self.data.focused_widget = *self.focus.last().unwrap();
            self.render_all(window_rect.clone(), graphics, !self.layout_valid);
            self.layout_valid = true;

            crate::widgets::run_animations(self, self.data.delta_time);

            let modals = self.data.modals.lock().unwrap();
            if let Some(m) = modals.last() {
                modals::modal::render_modal(&self.data.settings, m, &window_rect, graphics);
            }

        } else if let Some(controller) = &mut self.data.controller {
            let mut overlay = self.data.overlay.borrow_mut();
            for e in controller.get_actions(0) {
                overlay::handle_input(&mut overlay, None, Some(e));
            }
        }

        helper.request_redraw();
    }

    fn on_key_down(&mut self, _: &mut WindowHelper, virtual_key_code: Option<VirtualKeyCode>, _: KeyScancode) {
        if self.data.overlay_is_active() { return; }
        handle_input(self, virtual_key_code, None);
    }

    fn on_resize(&mut self, _: &mut WindowHelper, size_pixels: Vector2<u32>) { 
        self.data.win.size = size_pixels; 
        self.layout_valid = false;
    }
}

fn main() {
    let window = Window::new_fullscreen_borderless("Yaffe").unwrap();

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
    let state = YaffeState::new(overlay::OverlayWindow::new(settings.clone()), settings, q.clone());

    assets::initialize_asset_cache();

    let mut ui = widgets::WidgetTree::new(root, state);
    ui.focus(std::any::TypeId::of::<widgets::PlatformList>());

    window.run_loop(ui);
}

fn build_ui_tree(queue: Arc<RefCell<job_system::JobQueue>>) -> WidgetContainer {
    let mut root = WidgetContainer::root(widgets::Background::new(queue.clone()));
    root.add_child(widgets::PlatformList::new(queue.clone()), V2::new(0.25, 1.))
        .with_child(widgets::AppList::new(queue.clone()), V2::new(0.75, 1.))
            .add_child(widgets::SearchBar::new(queue.clone()), V2::new(1., 0.05))
            .add_child(widgets::Toolbar::new(queue.clone()), V2::new(1., 0.075))
            .add_child(widgets::InfoPane::new(queue.clone()), V2::new(0.33, 1.))
            .orientation(ContainerOrientation::Floating);
            
    root
}

fn shutdown(helper: &mut WindowHelper) {
    server::shutdown();
    #[cfg(windows)]
    unsafe { winapi::um::combaseapi::CoUninitialize() };
    helper.terminate_loop();
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
            "Exit Yaffe" => state.running = false, 
            "Shut Down" => { 
                state.running = false;
                crate::platform_layer::shutdown().display_failure("Failed to shut down", state); 
            },
            _ => panic!("Unknown menu option"),
        }
    }
}

fn handle_input(tree: &mut WidgetTree, code: Option<VirtualKeyCode>, button: Option<u16>) -> bool {
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
            SystemActions::ToggleOverlay => { /* Overlay handles this */ }
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