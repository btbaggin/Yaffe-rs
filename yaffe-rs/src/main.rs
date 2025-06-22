#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(maybe_uninit_array_assume_init, step_trait)]
#![allow(clippy::new_without_default)]
use std::cell::RefCell;
use std::rc::Rc;

/*
 * TODO
 * Search bar doesnt work well on plugins
 * render navigation stack
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
mod overlay_state;
mod overlay_window;
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
use logger::{error, PanicLogEntry};
use overlay_state::OverlayState;
use settings::SettingNames;
use state::{Tile, TileGroup, YaffeState};
use ui::DeferredAction;
use utils::{append_app_ext, LogicalPosition, LogicalSize, PhysicalRect, PhysicalSize, Rect, ScaleFactor, Transparent};
use widgets::*;
use winit::window::{WindowAttributes, WindowLevel};

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

    let process = Rc::new(RefCell::new(None));

    let yaffe_state = YaffeState::new(process.clone(), settings.clone(), queue.clone());
    let overlay_state = OverlayState::new(process.clone(), settings.clone());

    let overlay = ui::WidgetTree::<OverlayState, ()>::new(
        build_overlay_tree(),
        overlay_state,
        ui::WidgetId::of::<OverlayBackground>(),
    );
    let mut ui = ui::WidgetTree::<YaffeState, DeferredAction>::new(
        build_main_tree(),
        yaffe_state,
        ui::WidgetId::of::<PlatformList>(),
    );

    let input_map = input::get_input_map();
    let gamepad = os::initialize_gamepad().log_message_and_panic("Unable to initialize input");

    plugins::load_plugins(&mut ui.data, "./plugins");

    let main = WindowAttributes::default().with_title("Yaffe").with_visible(true);
    let overlay_att = WindowAttributes::default()
        .with_title("Overlay")
        .with_visible(false)
        .with_window_level(WindowLevel::AlwaysOnTop)
        .with_transparent(true)
        .with_decorations(false);

    let handlers = vec![
        windowing::WindowInfo::new(Rc::new(RefCell::new(overlay)), overlay_att, true),
        windowing::WindowInfo::new(Rc::new(RefCell::new(ui)), main, true),
    ];
    windowing::run_app(input_map, queue, handlers, notify, gamepad);
}

pub fn build_main_tree() -> ui::WidgetContainer<YaffeState, DeferredAction> {
    use ui::ContainerAlignment;

    let mut root = ui::WidgetContainer::root(Background::new());
    root.add_child(PlatformList::new(), LogicalSize::new(0.25, 1.), ContainerAlignment::Left)
        .with_child(AppList::new(), LogicalSize::new(0.75, 1.))
        .add_child(SearchBar::new(), LogicalSize::new(1., 0.05), ContainerAlignment::Top)
        .add_child(Toolbar::new(), LogicalSize::new(1., 0.075), ContainerAlignment::Bottom)
        .add_child(InfoPane::new(), LogicalSize::new(0.33, 1.), ContainerAlignment::Right);

    root
}

fn build_overlay_tree() -> ui::WidgetContainer<OverlayState, ()> { ui::WidgetContainer::root(OverlayBackground::new()) }
