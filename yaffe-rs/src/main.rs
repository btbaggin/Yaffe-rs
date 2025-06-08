#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(maybe_uninit_array_assume_init)]
use crate::logger::{error, PanicLogEntry};
use crate::utils::append_app_ext;
use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

/*
 * TODO
 * Search bar doesnt work well on plugins
 * on_frame_end? on_window_init?
*/

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");

mod assets;
mod data;
mod graphics;
mod input;
mod job_system;
mod logger;
mod modals;
mod os;
mod overlay;
mod platform;
mod plugins;
mod pooled_cache;
mod restrictions;
mod scraper;
mod settings;
mod state;
mod ui;
mod utils;
mod widgets;
mod windowing;
mod yaffe_window;

use graphics::Graphics;
use input::Actions;
use job_system::Job;
use settings::SettingNames;
use state::{Tile, TileGroup, YaffeState};
use ui::DeferredAction;
use utils::Transparent;
use utils::{LogicalPosition, LogicalSize, PhysicalRect, PhysicalSize, Rect, ScaleFactor};
use widgets::*;

const UPDATE_FILE_PATH: &str = "./yaffe-rs.update";

fn main() {
    logger::init();
    log_panics::init();

    //Check for and apply updates on startup
    if std::path::Path::new(UPDATE_FILE_PATH).exists() {
        let app = append_app_ext("./yaffe-rs");
        match crate::utils::yaffe_helper("update", &[UPDATE_FILE_PATH, &app]) {
            Ok(_) => return,
            Err(e) => error!("Updated file found, but unable to run updater {e:?}"),
        }
    }
    crate::data::init_database().log_message_and_panic("Unable to create database");

    let (queue, notify) = job_system::start_job_system();

    let settings = match settings::load_settings("./yaffe.settings") {
        Ok(settings) => settings,
        Err(e) => {
            logger::error!("Unable to load settings: {e:?}");
            settings::SettingsFile::default()
        }
    };
    logger::set_log_level(&settings.get_str(SettingNames::LoggingLevel));

    let q = Arc::new(Mutex::new(RefCell::new(queue)));
    let overlay = overlay::OverlayWindow::new(build_overlay_tree(), settings.clone());
    let state = YaffeState::new(overlay.clone(), settings, q.clone());

    let mut ui = ui::WidgetTree::new(build_main_tree(), state, std::any::TypeId::of::<PlatformList>());

    let input_map = input::get_input_map();
    let gamepad = os::initialize_gamepad().log_message_and_panic("Unable to initialize input");

    plugins::load_plugins(&mut ui.data, "./plugins");
    windowing::create_yaffe_windows(notify, q, gamepad, input_map, Rc::new(RefCell::new(ui)), overlay);
}

fn build_main_tree() -> ui::WidgetContainer {
    use ui::ContainerAlignment;

    let mut root = ui::WidgetContainer::root(Background::new());
    root.add_child(PlatformList::new(), LogicalSize::new(0.25, 1.), ContainerAlignment::Left)
        .with_child(AppList::new(), LogicalSize::new(0.75, 1.))
        .add_child(SearchBar::new(), LogicalSize::new(1., 0.05), ContainerAlignment::Top)
        .add_child(Toolbar::new(), LogicalSize::new(1., 0.075), ContainerAlignment::Bottom)
        .add_child(InfoPane::new(), LogicalSize::new(0.33, 1.), ContainerAlignment::Right);

    root
}

fn build_overlay_tree() -> ui::WidgetContainer { ui::WidgetContainer::root(Background::new()) }
