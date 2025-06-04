use crate::{YaffeState, Actions, DeferredAction, widget, LogicalPosition, LogicalSize, PhysicalSize, Rect};
use crate::widgets::AppTile;
use crate::logger::{PanicLogEntry, LogEntry, UserMessage};
use std::collections::HashMap;

const MARGIN: f32 = 0.1;

widget!(pub struct AppList {
    cached_platform: usize = 99999, 
    tiles: Vec<AppTile> = Vec::<AppTile>::new(),
    tiles_x: isize = 0,
    tiles_y: isize = 0,
    first_visible: HashMap<usize, isize> = HashMap::new(),
    tile_animation: f32 = 0.
});

impl crate::ui::Widget for AppList {
    fn action(&mut self, state: &mut YaffeState, action: &Actions, handler: &mut DeferredAction) -> bool {
        match action {
            Actions::Up => {
                let apps = state.settings.get_i32(crate::SettingNames::ItemsPerRow);
                self.update_position(state, apps, false, handler);
                true
            }
            Actions::Down => {
                let apps = state.settings.get_i32(crate::SettingNames::ItemsPerRow);
                self.update_position(state, apps, true, handler);
                true
            }
            Actions::Left => {
                self.update_position(state, 1, false, handler);
                true
            }
            Actions::Right => { 
                self.update_position(state, 1, true, handler);
                true 
            }
            Actions::Accept => {
                if let Some(exe) = state.get_executable() {
                   if crate::restrictions::rating_allowed(&exe.rating) ||
                      crate::restrictions::verify_restricted_action(state) {
                        start_game(state, handler)
                   }
                }
                true
            },
            Actions::Info => {
                handler.focus_widget(crate::get_widget_id!(crate::widgets::InfoPane));
                true
            },
            Actions::Filter => {
                handler.focus_widget(crate::get_widget_id!(crate::widgets::SearchBar));
                true
            }
            Actions::Back => {
                handler.revert_focus();
                true
            } 
            _ => false,
        }
    }

    fn got_focus(&mut self, _: &YaffeState) {
        self.tile_animation = 0.;
        let offset = crate::offset_of!(AppList => tile_animation);
        self.animate(offset, 1., crate::widgets::app_tile::ANIMATION_TIME);
    }

    fn render(&mut self, graphics: &mut crate::Graphics, state: &YaffeState) {
        self.update(state, graphics);

        let plat = state.get_platform();

        //Height needs to be based on image aspect * width
        let focused = crate::is_focused!(state);
        for i in 0..plat.apps.len() {
            if i == state.selected_app && focused { continue; }

            let tile = &mut self.tiles[i];
            //Only render tiles inside visible area
            if tile.intersects(&graphics.bounds) {
                tile.render(false, self.tile_animation, &plat.apps[i], graphics);
            }
        }

        if let Some(tile) = self.tiles.get_mut(state.selected_app) {
            tile.render(focused, self.tile_animation, &plat.apps[state.selected_app], graphics);
        }
    }
}
impl AppList {
    fn update(&mut self, state: &YaffeState, graphics: &mut crate::Graphics) {
        let scale_factor = graphics.scale_factor;
        let rect = graphics.bounds;
        //Check the length of our cache vs actual in case a game was added
        //to this platform while we were on it
        if self.cached_platform != state.selected_platform ||
            self.tiles.len() != state.get_platform().apps.len() {
            let platform = state.get_platform();
            self.tiles.clear();

            for i in 0..platform.apps.len() {
                self.tiles.push(AppTile::new(i));
            }

            self.cached_platform = state.selected_platform;
        }

        self.update_tiles(state, graphics, &rect, scale_factor);
    }

    fn update_tiles(&mut self, state: &YaffeState, graphics: &mut crate::Graphics, rect: &Rect, scale_factor: f32) {
        let platform = state.get_platform();

        //Calculate total size for inner list
        let first_visible = *self.first_visible.entry(state.selected_platform).or_insert(0);
        let margin_x = rect.width() * MARGIN;
        let margin_y = rect.height() * MARGIN;
        let list_rect = Rect::from_tuples((rect.left() + margin_x, rect.top() + margin_y), (rect.right() - margin_x, rect.bottom() - margin_y));

        //Get size each tile should try to stretch to
        let (tiles_x, tiles_y, ideal_tile_size) = self.get_ideal_tile_size(state,
            graphics, 
            !matches!(platform.kind, crate::state::GroupType::Emulator), 
            &list_rect.to_physical(scale_factor));

        self.tiles_x = tiles_x;
        self.tiles_y = tiles_y;
        let ideal_tile_size = ideal_tile_size.to_logical(scale_factor);

        let mut effective_i = 0;
        for tile in self.tiles.iter_mut() {
            tile.apply_filter(&state.search_info, &platform.apps);

            //Size each tile according to its aspect ratio and the ideal size
            AppList::size_individual_tile(state, graphics, tile, &ideal_tile_size);

            let x = (effective_i % self.tiles_x) as f32;
            let y = ((effective_i / self.tiles_x) - (first_visible / self.tiles_x)) as f32;

            let offset = (ideal_tile_size - tile.size) / 2.0;

            let position = LogicalPosition::new(ideal_tile_size.x * x + offset.x + list_rect.top_left().x, 
                                   ideal_tile_size.y * y + offset.y + list_rect.top_left().y);
            tile.position = position;

            if tile.is_visible() { effective_i += 1; }
        }
    }

    fn get_ideal_tile_size(&self, state: &YaffeState, graphics: &mut crate::Graphics, max: bool, rect: &crate::PhysicalRect) -> (isize, isize, PhysicalSize) {
        let mut width = 0.;
        let mut height = 0.;
        let mut tiles_x = 1;
        let mut tiles_y = 1;
        let mut bitmap_size = PhysicalSize::new(0., 0.);
        if !self.tiles.is_empty() {
            //Get widest boxart image
            let mut max_width = 0.;
            for exe in self.tiles.iter() {
                let size = exe.get_image_size(state, graphics);
                if size.x > max_width {
                    bitmap_size = size;
                    max_width = size.x;
                    if !max { break; }
                }
            }

            //Figure out size of each tile based on how many we want to fit
            if max_width > 0. {
                let menu_size = rect.size();

                if bitmap_size.x > bitmap_size.y {
                    let aspect = bitmap_size.y / bitmap_size.x;
                    width = menu_size.x / state.settings.get_i32(crate::SettingNames::ItemsPerRow) as f32;
                    height = aspect * width;
                } else {
                    let aspect = bitmap_size.x / bitmap_size.y;
                    height = menu_size.y / state.settings.get_i32(crate::SettingNames::ItemsPerColumn) as f32;
                    width = aspect * height;
                }
                tiles_x = (menu_size.x / width) as isize;
                tiles_y = (menu_size.y / height) as isize;
            }
        }

        (tiles_x, tiles_y, PhysicalSize::new(width, height))
    }

    fn size_individual_tile(state: &YaffeState, graphics: &mut crate::Graphics, tile: &mut AppTile, size: &LogicalSize) {
        let image = tile.get_image(state);
        let mut tile_size = *size;
        
        let bitmap_size = if let Some(i) = crate::assets::request_asset_image(graphics, &image) {
                i.size()
        } else {
            crate::assets::request_image(graphics, crate::assets::Images::Placeholder).unwrap().size()
        };
    
        //By default on the recents menu it chooses the widest game boxart (see pFindMax in GetTileSize)
        //We wouldn't want vertical boxart to stretch to the horizontal dimensions
        //This will scale boxart that is different aspect to fit within the tile_size.Height
        let bitmap_size = bitmap_size.to_logical(graphics.scale_factor);
        let real_aspect = bitmap_size.x / bitmap_size.y;
        let tile_aspect = tile_size.x / tile_size.y;
    
        //If an aspect is wider than it is tall, it is > 1
        //If the two aspect ratios are on other sides of one, it means we need to scale
        if f32::is_sign_positive(real_aspect - 1.) != f32::is_sign_positive(tile_aspect - 1.) {
            tile_size.x = tile_size.y * real_aspect;
        }
    
        tile.size = tile_size
    }

    fn increment_index(&self, index: usize, first_visible: isize, amount: i32, forward: bool) -> (usize, isize) {
        let mut index = index as isize;
        let old_index = index;
        let one = if forward { 1 } else { -1 };

        //Since certain roms could be filtered out,
	    //we will loop until we have incremented the proper amount of times
        let mut first_visible = first_visible;
        for _ in 0..amount {
	        //Move until we have found an unfiltered rom
            let mut new_index = index + one;
            while new_index >= 0 &&
                  (new_index as usize) < self.tiles.len() &&
                  !self.tiles[new_index as usize].is_visible() {
                new_index += one;
            }

            if new_index < 0 || new_index as usize >= self.tiles.len() { 
                //If we haven't moved the total amount we intended to
                //revert all changes. This will prevent it going to the last item when pressing down
                return (old_index as usize, first_visible);
            }
            index = new_index;
        }

        //Adjust first_visible index until our index is inside it
        while index < first_visible { first_visible -= self.tiles_x; }
        while index > (first_visible + (self.tiles_x * self.tiles_y) - 1) { first_visible += self.tiles_x; }
        assert!(first_visible >= 0);
        assert!(index >= 0);

        (index as usize, first_visible)
    }

    fn update_position(&mut self, state: &mut YaffeState, amount: i32, forward: bool, handler: &mut DeferredAction) {
        let old_index = state.selected_app;
        let first_visible = self.first_visible[&state.selected_platform];
        let (index, visible) = self.increment_index(state.selected_app, first_visible, amount, forward);
        if old_index != index {
            state.selected_app = index;
            *self.first_visible.get_mut(&state.selected_platform).unwrap() = visible;

            self.tile_animation = 0.;
            let offset = crate::offset_of!(AppList => tile_animation);
            self.animate(offset, 1., crate::widgets::app_tile::ANIMATION_TIME);

            if let crate::state::GroupType::Plugin = state.get_platform().kind {
                if visible + self.tiles_x * self.tiles_y >= self.tiles.len() as isize {
                    handler.load_plugin(crate::plugins::NavigationAction::Load);
                }
            }
        }
    }
}

fn start_game(state: &YaffeState, handler: &mut DeferredAction) {
    if let Some(exe) = state.get_executable() {

        if let Some(platform) = state.platforms.get(exe.platform_index) {
            let child = match platform.kind {
                crate::state::GroupType::Plugin => {
                    let plugin = platform.get_plugin(state).unwrap();
                    let process = plugin.borrow_mut().select(&exe.name, &exe.file);

                    match process {
                        None => {
                            handler.load_plugin(crate::plugins::NavigationAction::Load);
                            handler.focus_widget(crate::get_widget_id!(crate::widgets::AppList));
                            return;
                        },
                        Some(p) => p,
                    }
                },
                _ => {
                    let id = platform.id.unwrap();
                    //This should never fail since we got it from the database
                    let (path, args) = crate::data::PlatformInfo::get_info(id).log_message_and_panic("Platform not found");
                    crate::data::GameInfo::update_last_run(id, &exe.file).log("Unable to update game last run");

                    let mut process = &mut std::process::Command::new(path);
                    let exe_path = platform.get_rom_path().join(&exe.file);

                    process = process.arg(exe_path.to_str().unwrap());
                    if !args.is_empty() { process = process.args(args.split(' ')); }
                    process.spawn()
                }
            };

            if let Some(process) = child.display_failure_deferred("Unable to start game", handler) {
                let mut overlay = state.overlay.borrow_mut();
                overlay.set_process(process);
                //We could refresh so our recent games page updates, but I dont think that's desirable
            }
        }
    }
}