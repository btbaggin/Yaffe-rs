#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(maybe_uninit_array_assume_init, step_trait)]
#![allow(clippy::new_without_default)]
use std::cell::RefCell;
use std::rc::Rc;

/*
 * TODO
 * Search bar doesnt work well on plugins
 * allow reloading plugins?
 * Search bar needs to disappear
*/

const CARGO_PKG_VERSION: &str = env!("CARGO_PKG_VERSION");
const PLATFORM_LIST_ID: WidgetId = WidgetId::static_id(2);
const APP_LIST_ID: WidgetId = WidgetId::static_id(3);
const SEARCH_BAR_ID: WidgetId = WidgetId::static_id(4);

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
use ui::{DeferredAction, UiContainer, WidgetId};
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

    let settings = match settings::load_settings("./yaffe.settings", true) {
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

    let overlay = ui::WidgetTree::<OverlayState, ()>::new(build_overlay_tree(), overlay_state, WidgetId::static_id(1));
    let mut ui = ui::WidgetTree::<YaffeState, DeferredAction>::new(build_main_tree(), yaffe_state, PLATFORM_LIST_ID);

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
    windowing::run_app(queue, handlers, notify);
}

pub fn build_main_tree() -> UiContainer<YaffeState, DeferredAction> {
    use ui::ContainerSize;

    let mut root = UiContainer::row();
    root.background_image(crate::assets::Images::Background)
        .margin(0.)
        .add_child(PlatformList::new_with_id(PLATFORM_LIST_ID), ContainerSize::Percent(0.25))
        .with_child(UiContainer::column(), ContainerSize::Fill)
        .add_child(SearchBar::new_with_id(SEARCH_BAR_ID), ContainerSize::Percent(0.05))
        .add_child(AppList::new_with_id(APP_LIST_ID), ContainerSize::Fill)
        .add_child(Toolbar::new(), ContainerSize::Percent(0.05));

    root
}

use ui::ContainerSize;
fn build_overlay_tree() -> UiContainer<OverlayState, ()> {
    let mut root = UiContainer::row();
    root.add_child(OverlayBackground::new(), ContainerSize::Percent(1.));
    root
}
