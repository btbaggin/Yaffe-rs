use crate::assets::AssetKey;
use crate::graphics::Graphics;
use crate::input::Actions;
use crate::job_system::JobResult;
use crate::logger::LogEntry;
use crate::modals::{display_error, display_modal_raw, ModalSize, RestrictedMode, ScraperModal};
use crate::scraper::{GameScrapeResult, PlatformScrapeResult};
use crate::ui::{DeferredAction, WidgetTree};
use crate::widgets::InfoPane;
use crate::windowing::{WindowHandler, WindowHelper};
use crate::YaffeState;

impl WindowHandler for WidgetTree<YaffeState> {
    fn on_init(&mut self, graphics: &mut Graphics) { crate::assets::preload_assets(graphics); }

    fn on_fixed_update(&mut self, delta_time: f32, _: &mut WindowHelper) -> bool {
        //Check for any updates to the settings file
        // animations.process(self, delta_time);
        crate::settings::update_settings(&mut self.data.settings).log("Unable to retrieve updated settings");
        self.fixed_update(delta_time)
    }

    fn on_frame_begin(&mut self, graphics: &mut Graphics, jobs: Vec<JobResult>) { process_jobs(self, graphics, jobs); }

    fn on_frame(&mut self, graphics: &mut Graphics) -> bool {
        if !self.data.is_overlay_active() {
            //Update the platform and emulator list from database
            if self.data.refresh_list {
                crate::platform::get_database_info(&mut self.data);
                self.data.refresh_list = false;
            }

            graphics.cache_settings(&self.data.settings);

            self.render(graphics);
        }

        let cache_size = self.data.settings.get_i32(crate::settings::SettingNames::AssetCacheSizeMb) as usize;
        crate::assets::clear_old_cache(graphics, cache_size);

        self.data.running
    }

    fn on_input(&mut self, _: &mut WindowHelper, action: &Actions) -> bool {
        if self.data.is_overlay_active() {
            return false;
        }

        match action {
            Actions::ShowMenu => {
                if !self.is_modal_open() {
                    let items = vec![
                        "Scan For New Roms".to_string(),
                        "Add Emulator".to_string(),
                        match self.data.restricted_mode {
                            RestrictedMode::On(_) => "Disable Restricted Mode".to_string(),
                            RestrictedMode::Off => "Enable Restricted Mode".to_string(),
                        },
                        "Settings".to_string(),
                        "Exit Yaffe".to_string(),
                        "Shut Down".to_string(),
                    ];

                    let list = crate::modals::MenuModal::from(items);
                    crate::modals::display_modal_raw(self, "Menu", None, list, ModalSize::Third);
                    true
                } else {
                    false
                }
            }
            Actions::ToggleOverlay => {
                false /* Overlay handles this */
            }
            _ => {
                let mut handler = DeferredAction::new();
                let result = self.action(action, &mut handler);
                handler.resolve(self);
                result
            }
        }
    }

    fn on_stop(&mut self) { crate::plugins::unload(&mut self.data.plugins); }
}

fn process_jobs(ui: &mut WidgetTree<YaffeState>, graphics: &mut Graphics, job_results: Vec<JobResult>) {
    for r in job_results {
        match r {
            JobResult::LoadImage { data, dimensions, key } => {
                let mut map = graphics.asset_cache.borrow_mut();
                let asset_slot = crate::assets::get_asset_slot(&mut map, &key);
                asset_slot.set_data(data, dimensions);
            }
            JobResult::SearchGame(result) => match result {
                Ok(result) => {
                    if let Some(game) = result.get_exact() {
                        crate::platform::insert_game(&mut ui.data, &game.info, game.boxart.clone());
                    } else if result.count > 0 {
                        let items = result.results;
                        let content = ScraperModal::from(items, false, build_game_info);
                        display_modal_raw(
                            ui,
                            &format!("Select Game: {}", result.request),
                            None,
                            content,
                            ModalSize::Half,
                        );
                    }
                }
                Err(e) => display_error(ui, format!("Error occured while searching games: {e:?}")),
            },
            JobResult::SearchPlatform(result) => match result {
                Ok(result) => {
                    if let Some(platform) = result.get_exact() {
                        crate::platform::insert_platform(&mut ui.data, &platform.info);
                    } else if result.count > 0 {
                        let items = result.results;
                        let content = ScraperModal::from(items, true, build_platform_info);
                        display_modal_raw(ui, "Select Platform", None, content, ModalSize::Half);
                    }
                }
                Err(e) => display_error(ui, format!("Error occured while searching platforms: {e:?}")),
            },
            _ => {}
        }
    }
}

fn build_platform_info(item: &PlatformScrapeResult) -> InfoPane<YaffeState> {
    let attributes = vec![];
    InfoPane::from(AssetKey::Url(item.boxart.clone()), item.overview.clone(), attributes)
}

fn build_game_info(item: &GameScrapeResult) -> InfoPane<YaffeState> {
    let attributes = vec![
        ("Players".to_string(), item.info.players.to_string()),
        ("Rating".to_string(), item.info.rating.clone()),
        ("Released".to_string(), item.info.released.clone()),
    ];
    InfoPane::from(AssetKey::Url(item.boxart.clone()), item.info.overview.clone(), attributes)
}
