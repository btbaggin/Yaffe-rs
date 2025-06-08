use crate::YaffeState;
use crate::windowing::{WindowHandler, WindowHelper};
use crate::ui::{WidgetTree, DeferredAction, ModalResult, AnimationManager, display_modal};
use crate::modals::{SetRestrictedModal, SettingsModal, PlatformDetailModal, ListModal, on_add_platform_close, on_settings_close};
use crate::input::Actions;
use crate::job_system::JobResult;
use crate::graphics::Graphics;
use crate::logger::{UserMessage, PanicLogEntry, LogEntry};
use crate::restrictions::{RestrictedMode, on_restricted_modal_close};


impl WindowHandler for WidgetTree {
    fn on_fixed_update(&mut self, _: &mut WindowHelper) -> bool {
        //Clear any assets that haven't been requested in a long time

        //Check for any updates to the settings file
        crate::settings::update_settings(&mut self.data.settings).log("Unable to retrieve updated settings")
    }

    fn on_frame_begin(&mut self, graphics: &mut Graphics, jobs: &mut Vec<JobResult>) {
        crate::assets::preload_assets(graphics);
        process_jobs(&mut self.data, graphics, jobs);
    }

    fn on_frame(&mut self, graphics: &mut Graphics) -> bool {
        if !self.data.overlay.borrow().is_active() {
            
            let window_rect = graphics.bounds;

            //Update the platform and emulator list from database
            if self.data.refresh_list {
                crate::platform::get_database_info(&mut self.data);
                self.data.refresh_list = false;
            }

            graphics.cache_settings(&self.data.settings);
            self.data.focused_widget = *self.focus.last().unwrap();
            self.render_all(graphics);

            //Render modal last, on top of everything
            let modals = self.data.modals.lock().unwrap();
            if let Some(m) = modals.last() {
                // Render calls will modify the bounds, so we must reset it
                graphics.bounds = window_rect;
                crate::ui::render_modal(m, graphics);
            }

            if !self.data.toasts.is_empty() {
                // Render calls will modify the bounds, so we must reset it
                graphics.bounds = window_rect;
                crate::ui::render_toasts(&self.data.toasts, graphics);
            }
        }

        let cache_size = self.data.settings.get_i32(crate::settings::SettingNames::AssetCacheSizeMb) as usize;
        crate::assets::clear_old_cache(graphics, cache_size);

        self.data.running
    }

    fn on_input(&mut self, animations: &mut AnimationManager, helper: &mut WindowHelper, action: &Actions) -> bool {
        if self.data.overlay.borrow().is_active() { return false; }

        match action {
            Actions::ShowMenu => {
                if !crate::ui::is_modal_open(&self.data) {
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
        
                    let list = Box::new(crate::modals::ListModal::new(items));
                    crate::ui::display_modal(&mut self.data, "Menu", None, list, Some(on_menu_close));
                    true
                } else {
                    false
                }
            },
            Actions::ToggleOverlay => { false /* Overlay handles this */ }
            _ => {
                let mut handler = DeferredAction::new();
                let result = if !crate::ui::is_modal_open(&self.data) {
                    let focus = self.focus.last().log_and_panic();
        
                    self.root.action(&mut self.data, animations, action, focus, &mut handler)
                } else {
                    crate::ui::update_modal(&mut self.data, helper, action, &mut handler);
                    true
                };
                handler.resolve(self, animations);
                result
            }
        }
    }

    fn on_stop(&mut self) {
        crate::plugins::unload(&mut self.data.plugins);
    }

    fn get_ui(&mut self) -> &mut crate::ui::WidgetContainer {
        &mut self.root
    }
}

fn on_menu_close(state: &mut YaffeState, result: ModalResult, content: &dyn crate::ui::ModalContent, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let list_content = content.as_any().downcast_ref::<ListModal<String>>().unwrap();
        
        match &list_content.get_selected()[..] {
            "Add Emulator" => {
                let content = Box::new(PlatformDetailModal::emulator());
                display_modal(state, "New Emulator", Some("Confirm"), content, Some(on_add_platform_close));
            },
            "Settings" => {
                let content = Box::new(SettingsModal::new(&state.settings));
                display_modal(state, "Settings", Some("Confirm"), content, Some(on_settings_close));
            },
            "Disable Restricted Mode" => {
                let content = Box::new(SetRestrictedModal::new());
                display_modal(state, "Restricted Mode", Some("Set passcode"), content, Some(on_restricted_modal_close))
            },
            "Enable Restricted Mode" => {
                let content = Box::new(SetRestrictedModal::new());
                display_modal(state, "Restricted Mode", Some("Set passcode"), content, Some(on_restricted_modal_close))
            },
            "Scan For New Roms" => crate::platform::scan_new_files(state),
            "Exit Yaffe" => state.running = false, 
            "Shut Down" => { 
                if crate::os::shutdown().display_failure("Failed to shut down", state).is_some() {
                    state.running = false;
                }
            },
            _ => panic!("Unknown menu option"),
        }
    }
}

fn process_jobs(state: &mut YaffeState, graphics: &mut Graphics, job_results: &mut Vec<JobResult>) {
    crate::job_system::process_results(job_results, |j| 
        matches!(j, JobResult::LoadImage { .. } | JobResult::SearchGame(_) | JobResult::SearchPlatform(_)), 
    |result| {
        match result {
            JobResult::LoadImage { data, dimensions, key } => {
                let mut map = graphics.asset_cache.borrow_mut();
                let asset_slot = crate::assets::get_asset_slot(&mut map, &key);
                asset_slot.set_data(data, dimensions);
            },
            JobResult::SearchGame(result) => {
                use crate::modals::GameScraperModal;
                if let Some(game) = result.get_exact() {
                    crate::platform::insert_game(state, &game.info, game.boxart.clone());

                } else if result.count > 0 {
                    let mut items = vec!();
                    for i in result.results {
                        items.push(i);
                    }
                    
                    let content = GameScraperModal::new(items);
                    display_modal(state, &format!("Select Game: {}", result.request), None, Box::new(content), Some(crate::modals::on_game_found_close));
                }

                state.toasts.remove(&result.id);
            },
            JobResult::SearchPlatform(result) => {
                use crate::modals::PlatformScraperModal;
                if let Some(platform) = result.get_exact() {
                    crate::platform::insert_platform(state, &platform.info);

                } else if result.count > 0 {
                    let mut items = vec!();
                    for i in result.results {
                        items.push(i);
                    }
                    
                    let content = PlatformScraperModal::new(items);
                    display_modal(state, "Select Platform", None, Box::new(content), Some(crate::modals::on_platform_found_close));
                }

                state.toasts.remove(&result.id);
            }
            _ => {},
        }
    });
}