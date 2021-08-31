#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
use std::time::Instant;
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use speedy2d::shape::Rectangle;
use speedy2d::dimen::Vector2;
pub use crate::settings::SettingNames;
use crate::logger::{UserMessage, LogEntry};

#[macro_use]
extern crate dlopen_derive;

type V2 = Vector2<f32>;

/*
    TODO:
    button remapping?
    scale factor
    move yaffe-service logic here
    move api key to settings file
*/

pub mod colors {
    use speedy2d::color::Color;
    pub const MENU_BACKGROUND: Color = Color::from_rgba(0.2, 0.2, 0.2, 0.7);
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

mod widgets;
mod assets;
mod database;
mod modals;
mod platform;
mod platform_layer;
mod overlay;
mod restrictions;
mod game_db;
mod job_system;
mod logger;
mod settings;
mod windowing;
mod input;
mod plugins;
use windowing::{Rect, Transparent};
use widgets::*;
use overlay::OverlayWindow;
use restrictions::RestrictedMode;
use modals::{display_modal};
use job_system::{JobQueue, JobType, RawDataPointer};
use input::Actions;

pub struct Platform {
    id: Option<i64>,
    name: String,
    path: String,
    apps: Vec<Executable>,
    kind: platform::PlatformType,
    plugin_index: usize,
}

pub struct Executable {
    file: String,
    name: String,
    description: String,
    rating: platform::Rating,
    players: u8,
    platform_index: usize,
    boxart: Rc<RefCell<assets::AssetSlot>>,
    banner: Rc<RefCell<assets::AssetSlot>>,
}

pub struct YaffeState {
    overlay: Rc<RefCell<OverlayWindow>>,
    selected_platform: usize,
    selected_app: usize,
    platforms: Vec<Platform>,
    plugins: Vec<RefCell<plugins::Plugin>>,
    focused_widget: widgets::WidgetId,
    modals: std::sync::Mutex<Vec<modals::Modal>>,
    queue: Arc<RefCell<job_system::JobQueue>>,
    search_info: widgets::SearchInfo,
    restricted_mode: RestrictedMode,
    restricted_last_approve: Option<Instant>,
    refresh_list: bool,
    settings: settings::SettingsFile,
    running: bool,
}
impl YaffeState {
    fn new(overlay: Rc<RefCell<OverlayWindow>>, 
           settings: settings::SettingsFile, 
           queue: Arc<RefCell<job_system::JobQueue>>) -> YaffeState {
        YaffeState {
            overlay: overlay,
            selected_platform: 0,
            selected_app: 0,
            platforms: vec!(),
            plugins: vec!(),
            search_info: SearchInfo::new(),
            focused_widget: get_widget_id!(widgets::PlatformList),
            restricted_mode: RestrictedMode::Off,
            restricted_last_approve: None,
            modals: std::sync::Mutex::new(vec!()),
            queue: queue,
            refresh_list: true,
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

    fn is_widget_focused(&self, widget: &impl FocusableWidget) -> bool {
        self.focused_widget == widget.get_id()
    }
}

impl windowing::WindowHandler for WidgetTree {
    fn on_frame(&mut self, graphics: &mut speedy2d::Graphics2D, delta_time: f32, size: Vector2<u32>) -> bool {
        if let Err(e) = settings::update_settings(&mut self.data.settings) {
            logger::log_entry_with_message(logger::LogTypes::Warning, e, "Unable to retrieve updated settings");
        }

        assets::load_texture_atlas(graphics);

        if !self.data.overlay.borrow().is_active() {
            let window_rect = Rectangle::from_tuples((0., 0.), (size.x as f32, size.y as f32));

            if self.data.refresh_list {
                platform::get_database_info(&mut self.data);
                self.data.refresh_list = false;
            }

            self.data.focused_widget = *self.focus.last().unwrap();
            self.render_all(window_rect.clone(), graphics, delta_time, !self.layout_valid);
            self.layout_valid = true;

            crate::widgets::animations::run_animations(self, delta_time);

            let modals = self.data.modals.lock().unwrap();
            if let Some(m) = modals.last() {
                modals::modal::render_modal(&self.data.settings, m, &window_rect, graphics);
            }

        }

        self.data.running
    }

    fn on_input(&mut self, helper: &mut windowing::WindowHelper, action: &Actions) -> bool {
        if self.data.overlay.borrow().is_active() { return false; }

        match action {
            Actions::ShowMenu => {
                let mut l = Box::new(modals::ListModal::new(None));
                l.add_item(String::from("Exit Yaffe"));
                l.add_item(String::from("Shut Down"));
                l.add_item(String::from("Settings"));
                match self.data.restricted_mode {
                    RestrictedMode::On(_) => l.add_item(String::from("Disable Restricted Mode")),
                    RestrictedMode::Off => l.add_item(String::from("Enable Restricted Mode")),
                    RestrictedMode::Pending => {},
                }
                l.add_item(String::from("Add Emulator"));
                l.add_item(String::from("Add Application"));
    
                display_modal(&mut self.data, "Menu", None, l, modals::ModalSize::Third, Some(on_menu_close));
                return true;
            },
            Actions::ToggleOverlay => { return false; /* Overlay handles this */ }
            _ => {
                if !modals::modal::is_modal_open(&self.data) {
                    let mut handler = DeferredAction::new();
                    let focus = self.focus.last().log_if_fail();
        
                    self.root.action(&mut self.data, &action, focus, &mut handler);
                    handler.resolve(self);
                } else {
                    modals::modal::update_modal(&mut self.data, helper, &action);
                }
                return true;
            }
        }
    }

    fn on_stop(&mut self) {
        plugins::unload(&mut self.data.plugins);
    }

    fn on_resize(&mut self, _: u32, _: u32) { 
        self.layout_valid = false;
    }

    //TODO check settings in on_fixed_update?

    fn is_window_dirty(&self) -> bool {
        self.anims.len() > 0
    }
}

fn main() {
    logger::initialize_log();
    let (queue, notify) = job_system::start_job_system();

    let (settings, plugins) = match settings::load_settings("./settings.txt") {
        Ok(settings) => settings,
        Err(e) => {
            logger::log_entry(logger::LogTypes::Error, e);
            (settings::SettingsFile::default(), settings::PluginSettings::default())
        },
    };

    let q = Arc::new(RefCell::new(queue));
    let root = build_ui_tree(q.clone());
    let overlay = overlay::OverlayWindow::new(settings.clone());
    let state = YaffeState::new(overlay.clone(), settings, q.clone());

    assets::initialize_asset_cache();

    let mut ui = widgets::WidgetTree::new(root, state);
    ui.focus(std::any::TypeId::of::<widgets::PlatformList>());
 
    let input_map = input::get_input_map();
    let gamepad = platform_layer::initialize_gamepad().log_message_if_fail("Unable to initialize input");

    plugins::load_plugins(&mut ui.data, "./plugins", plugins);
    windowing::create_yaffe_windows(notify, gamepad, input_map, Rc::new(RefCell::new(ui)), overlay);
}

fn build_ui_tree(queue: Arc<RefCell<job_system::JobQueue>>) -> WidgetContainer {
    let mut root = WidgetContainer::root(widgets::Background::new(queue.clone()));
    root.add_child(widgets::PlatformList::new(queue.clone()), V2::new(0.25, 1.), ContainerAlignment::Left)
        .with_child(widgets::AppList::new(queue.clone()), V2::new(0.75, 1.))
            .add_child(widgets::SearchBar::new(queue.clone()), V2::new(1., 0.05), ContainerAlignment::Top)
            .add_child(widgets::Toolbar::new(queue.clone()), V2::new(1., 0.075), ContainerAlignment::Bottom)
            .add_child(widgets::InfoPane::new(queue.clone()), V2::new(0.33, 1.), ContainerAlignment::Right);
            
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
            "Exit Yaffe" => state.running = false, 
            "Shut Down" => { 
                state.running = false;
                crate::platform_layer::shutdown().display_failure("Failed to shut down", state); 
            },
            _ => panic!("Unknown menu option"),
        }
    }
}