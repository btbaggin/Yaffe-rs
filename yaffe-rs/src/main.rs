#![cfg_attr(not(debug_assertions), windows_subsystem = "windows")]
#![feature(maybe_uninit_array_assume_init)]
#![feature(assert_matches)]
use std::rc::Rc;
use std::sync::Arc;
use std::cell::RefCell;
use std::sync::Mutex;
use platform::scan_new_files;
use crate::logger::{UserMessage, PanicLogEntry, LogEntry, error};
use crate::assets::AssetPathType;

/* 
 * TODO
 * Better ORM stuff
 * Auto DB migration with type introspection?
 * Add horizontal container and vertical container
 * use git for update instead of drive
 * custom modals for platform and game scraping
 * display more info on info pane
*/

const CARGO_PKG_VERSION: &'static str = env!("CARGO_PKG_VERSION");
const UPDATE_FILE_PATH: &'static str = "./yaffe-rs.update";

#[macro_use]
extern crate dlopen_derive;

mod widgets;
mod assets;
mod data;
mod modals;
mod platform;
mod platform_layer;
mod overlay;
mod restrictions;
mod scraper;
mod job_system;
mod logger;
mod settings;
mod windowing;
mod input;
mod plugins;
mod ui_control;
mod utils;
mod pooled_cache;
mod graphics;

use utils::Transparent;
use widgets::*;
use overlay::OverlayWindow;
use restrictions::RestrictedMode;
use modals::display_modal;
use job_system::{JobType, RawDataPointer};
use input::Actions;
pub use graphics::Graphics;
pub use crate::settings::SettingNames;
pub use utils::{LogicalPosition, LogicalSize, PhysicalPosition, PhysicalSize, Rect, PhysicalRect, ScaleFactor};

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
    players: u8,
    platform_index: usize,
    boxart: AssetPathType,
}

pub struct YaffeState {
    overlay: Rc<RefCell<OverlayWindow>>,
    selected_platform: usize,
    selected_app: usize,
    platforms: Vec<Platform>,
    plugins: Vec<RefCell<plugins::Plugin>>,
    focused_widget: widgets::WidgetId,
    modals: Mutex<Vec<modals::Modal>>,
    queue: job_system::ThreadSafeJobQueue,
    search_info: widgets::SearchInfo,
    restricted_mode: RestrictedMode,
    refresh_list: bool,
    settings: settings::SettingsFile,
    running: bool,
    update_timer: f32,
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
            update_timer: 0.,
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
    fn on_fixed_update(&mut self, _: &mut crate::windowing::WindowHelper, delta_time: f32) -> bool {
        //Clear any assets that haven't been requested in a long time
        crate::assets::clear_old_cache(&self.data);

        //Check for updates once every hour if it hasnt been applied already
        if self.data.update_timer != f32::NAN {
            self.data.update_timer -= delta_time;
            if self.data.update_timer < 0. {
                let lock = self.data.queue.lock().log_and_panic();
                let mut queue = lock.borrow_mut();

                let applied = scraper::check_for_updates(&mut queue).log("Error checking for updates");
                if applied { self.data.update_timer = f32::NAN; }
                else { self.data.update_timer = 60. * 60.; }
            }
        }

        //Check for any updates to the settings file
        settings::update_settings(&mut self.data.settings).log("Unable to retrieve updated settings")
    }

    fn on_frame(&mut self, graphics: &mut speedy2d::Graphics2D, delta_time: f32, size: PhysicalSize, scale_factor: f32) -> bool {
        assets::preload_assets(graphics);

        if !self.data.overlay.borrow().is_active() {
            let window_rect = Rect::new(LogicalPosition::new(0., 0.), size.to_logical(scale_factor));

            //Update the platform and emulator list from database
            if self.data.refresh_list {
                platform::get_database_info(&mut self.data);
                self.data.refresh_list = false;
            }

            let mut graphics = Graphics { graphics, queue: Some(self.data.queue.clone()), scale_factor, bounds: window_rect.clone(), delta_time };
            self.data.focused_widget = *self.focus.last().unwrap();
            self.render_all(&mut graphics);

            crate::widgets::animations::run_animations(self, delta_time);

            //Render modal last, on top of everything
            let modals = self.data.modals.lock().unwrap();
            if let Some(m) = modals.last() {
                graphics.bounds = window_rect;
                modals::render_modal(&self.data.settings, m, &mut graphics);
            }
        }

        self.data.running
    }

    fn on_input(&mut self, helper: &mut windowing::WindowHelper, action: &Actions) -> bool {
        if self.data.overlay.borrow().is_active() { return false; }

        match action {
            Actions::ShowMenu => {
                //TODO this modal can stack which I dont like
                let mut items = vec!();
                items.push(String::from("Scan For New Roms"));
                items.push(String::from("Add Emulator"));
                match self.data.restricted_mode {
                    RestrictedMode::On(_) => items.push(String::from("Disable Restricted Mode")),
                    RestrictedMode::Off => items.push(String::from("Enable Restricted Mode")),
                }
                items.push(String::from("Settings"));
                items.push(String::from("Exit Yaffe"));
                items.push(String::from("Shut Down"));
    
                let l = Box::new(modals::ListModal::new(items));
                display_modal(&mut self.data, "Menu", None, l, Some(on_menu_close));
                true
            },
            Actions::ToggleOverlay => { false /* Overlay handles this */ }
            _ => {
                let mut handler = DeferredAction::new();
                let result = if !modals::is_modal_open(&self.data) {
                    let focus = self.focus.last().log_and_panic();
        
                    self.root.action(&mut self.data, &action, focus, &mut handler)
                } else {
                    modals::update_modal(&mut self.data, helper, &action, &mut handler);
                    true
                };
                handler.resolve(self);
                result
            }
        }
    }

    fn on_stop(&mut self) {
        plugins::unload(&mut self.data.plugins);
    }

    fn on_resize(&mut self, _: u32, _: u32) { 
        self.invalidate()
    }

    fn is_window_dirty(&self) -> bool {
        self.needs_new_frame()
    }
}

fn main() {
    logger::init();
    
    //Check for and apply updates on startup
    if std::path::Path::new(UPDATE_FILE_PATH).exists() {
        match platform_layer::update() { 
            Ok(_) => return,
            Err(e) => error!("Updated file found, but unable to run updater {:?}", e),
        }
    }

    let (queue, notify) = job_system::start_job_system();

    let settings = match settings::load_settings("./yaffe.settings") {
        Ok(settings) => settings,
        Err(e) => {
            logger::error!("Unable to load settings: {:?}", e);
            settings::SettingsFile::default()
        },
    };
    logger::set_log_level(&settings.get_str(SettingNames::LoggingLevel));

    let q = Arc::new(Mutex::new(RefCell::new(queue)));
    let root = build_ui_tree();
    let overlay = overlay::OverlayWindow::new(settings.clone());
    let state = YaffeState::new(overlay.clone(), settings, q.clone());

    assets::initialize_asset_cache();

    let mut ui = widgets::WidgetTree::new(root, state);
    ui.focus(std::any::TypeId::of::<widgets::PlatformList>());
 
    let input_map = input::get_input_map();
    let gamepad = platform_layer::initialize_gamepad().log_message_and_panic("Unable to initialize input");

    plugins::load_plugins(&mut ui.data, "./plugins");
    windowing::create_yaffe_windows(notify, gamepad, input_map, Rc::new(RefCell::new(ui)), overlay);
}

fn build_ui_tree() -> WidgetContainer {
    let mut root = WidgetContainer::root(widgets::Background::new());
    root.add_child(widgets::PlatformList::new(), LogicalSize::new(0.25, 1.), ContainerAlignment::Left)
        .with_child(widgets::AppList::new(), LogicalSize::new(0.75, 1.))
            .add_child(widgets::SearchBar::new(), LogicalSize::new(1., 0.05), ContainerAlignment::Top)
            .add_child(widgets::Toolbar::new(), LogicalSize::new(1., 0.075), ContainerAlignment::Bottom)
            .add_child(widgets::InfoPane::new(), LogicalSize::new(0.33, 1.), ContainerAlignment::Right);
            
    root
}

fn on_menu_close(state: &mut YaffeState, result: modals::ModalResult, content: &Box<dyn modals::ModalContent>, _: &mut crate::DeferredAction) {
    if let modals::ModalResult::Ok = result {
        let list_content = content.as_any().downcast_ref::<modals::ListModal<String>>().unwrap();
        
        match &list_content.get_selected()[..] {
            "Add Emulator" => {
                let content = Box::new(modals::PlatformDetailModal::emulator());
                display_modal(state, "New Emulator", Some("Confirm"), content, Some(modals::on_add_platform_close));
            },
            "Settings" => {
                let content = Box::new(modals::SettingsModal::new(&state.settings));
                display_modal(state, "Settings", Some("Confirm"), content, Some(modals::on_settings_close));
            },
            "Disable Restricted Mode" => {
                let content = Box::new(modals::SetRestrictedModal::new());
                display_modal(state, "Restricted Mode", Some("Set passcode"), content, Some(restrictions::on_restricted_modal_close))
            },
            "Enable Restricted Mode" => {
                let content = Box::new(modals::SetRestrictedModal::new());
                display_modal(state, "Restricted Mode", Some("Set passcode"), content, Some(restrictions::on_restricted_modal_close))
            },
            "Scan For New Roms" => {
                scan_new_files(state);
            },
            "Exit Yaffe" => state.running = false, 
            "Shut Down" => { 
                if let Some(_) = crate::platform_layer::shutdown().display_failure("Failed to shut down", state) {
                    state.running = false;
                }
            },
            _ => panic!("Unknown menu option"),
        }
    }
}