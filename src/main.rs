#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(maybe_uninit_array_assume_init)]
use std::rc::Rc;
use std::sync::{Arc, Mutex};
use std::cell::RefCell;
use platform::scan_new_files;
use crate::logger::{PanicLogEntry, error};
use crate::assets::AssetKey;
use yaffe_lib::UPDATE_FILE_PATH;

/* 
 * TODO
 * Fix selected_app when moving between large things
*/

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

mod widgets;
mod assets;
mod data;
mod modals;
mod platform;
mod os;
mod overlay;
mod restrictions;
mod scraper;
mod job_system;
mod logger;
mod settings;
mod windowing;
mod input;
mod plugins;
mod utils;
mod pooled_cache;
mod graphics;
mod ui;
mod yaffe_window;

use ui::DeferredAction;
use utils::Transparent;
use widgets::*;
use overlay::OverlayWindow;
use restrictions::RestrictedMode;
use job_system::Job;
use input::Actions;
use graphics::Graphics;
use settings::SettingNames;
use utils::{LogicalPosition, LogicalSize, PhysicalSize, Rect, PhysicalRect, ScaleFactor};

pub struct Platform {
    id: Option<i64>,
    name: String,
    apps: Vec<Executable>,
    kind: platform::PlatformType,
    plugin_index: usize,
}

pub struct Executable {
    file: String,
    name: String,
    description: String,
    rating: platform::Rating,
    released: String,
    players: u8,
    platform_index: usize,
    boxart: AssetKey,
}

pub struct YaffeState {
    overlay: Rc<RefCell<OverlayWindow>>,
    selected_platform: usize,
    selected_app: usize,
    platforms: Vec<Platform>,
    plugins: Vec<RefCell<plugins::Plugin>>,
    focused_widget: ui::WidgetId,
    modals: Mutex<Vec<ui::Modal>>,
    queue: job_system::ThreadSafeJobQueue,
    search_info: SearchInfo,
    restricted_mode: RestrictedMode,
    refresh_list: bool,
    settings: settings::SettingsFile,
    running: bool,
}
impl YaffeState {
    fn new(overlay: Rc<RefCell<OverlayWindow>>, 
           settings: settings::SettingsFile, 
           queue: job_system::ThreadSafeJobQueue) -> YaffeState {
        YaffeState {
            overlay,
            selected_platform: 0,
            selected_app: 0,
            platforms: vec!(),
            plugins: vec!(),
            search_info: SearchInfo::new(),
            focused_widget: get_widget_id!(widgets::PlatformList),
            restricted_mode: RestrictedMode::Off,
            modals: Mutex::new(vec!()),
            queue,
            refresh_list: true,
            settings,
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
}


fn main() {
    logger::init();
    
    //Check for and apply updates on startup
    if std::path::Path::new(UPDATE_FILE_PATH).exists() {
        match os::update() { 
            Ok(_) => return,
            Err(e) => error!("Updated file found, but unable to run updater {:?}", e),
        }
    }
    crate::data::init_database().log_message_and_panic("Unable to create database");

    let (queue, notify) = job_system::start_job_system();

    let settings = match settings::load_settings("./yaffe.settings") {
        Ok(settings) => settings,
        Err(e) => {
            logger::error!("Unable to load settings: {:?}", e);
            settings::SettingsFile::default()
        },
    };
    logger::set_log_level(&settings.get_str(SettingNames::LoggingLevel));

    let animation = Rc::new(RefCell::new(ui::AnimationManager::new()));
    let q = Arc::new(Mutex::new(RefCell::new(queue)));
    let root = build_ui_tree(animation.clone());
    let overlay = overlay::OverlayWindow::new(settings.clone(), q.clone());
    let state = YaffeState::new(overlay.clone(), settings, q.clone());

    assets::initialize_asset_cache();

    let mut ui = ui::WidgetTree::new(root, animation, state);
    ui.focus(std::any::TypeId::of::<PlatformList>());
 
    let input_map = input::get_input_map();
    let gamepad = os::initialize_gamepad().log_message_and_panic("Unable to initialize input");

    plugins::load_plugins(&mut ui.data, "./plugins");
    windowing::create_yaffe_windows(notify, q, gamepad, input_map, Rc::new(RefCell::new(ui)), overlay);
}

fn build_ui_tree(animation: Rc<RefCell<ui::AnimationManager>>) -> ui::WidgetContainer {
    use ui::ContainerAlignment;

    let mut root = ui::WidgetContainer::root(Background::new(animation.clone()));
    root.add_child(PlatformList::new(animation.clone()), LogicalSize::new(0.25, 1.), ContainerAlignment::Left)
        .with_child(AppList::new(animation.clone()), LogicalSize::new(0.75, 1.))
            .add_child(SearchBar::new(animation.clone()), LogicalSize::new(1., 0.05), ContainerAlignment::Top)
            .add_child(Toolbar::new(animation.clone()), LogicalSize::new(1., 0.075), ContainerAlignment::Bottom)
            .add_child(InfoPane::new(animation), LogicalSize::new(0.33, 1.), ContainerAlignment::Right);
            
    root
}

