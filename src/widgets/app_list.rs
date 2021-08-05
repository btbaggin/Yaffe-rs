use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::{YaffeState, Actions, DeferredAction, widget, V2};
use crate::widgets::AppTile;
use crate::Rect;
use crate::logger::{LogEntry, UserMessage};

const APPS_PER_ROW: usize = 4;
const MARGIN: f32 = 0.1;

widget!(pub struct AppList {
    cached_platform: usize = 99999, 
    tiles: Vec<AppTile> = Vec::<AppTile>::new(),
    tiles_x: isize = 0,
    tiles_y: isize = 0,
    first_visible: isize = 0
});

impl super::Widget for AppList {
    fn action(&mut self, state: &mut YaffeState, action: &Actions, handler: &mut DeferredAction) -> bool {
        match action {
            Actions::Up => {
                handler.animate_placeholder(crate::widgets::app_tile::ANIMATION_TIME);
                let (index, visible) = self.increment_index(state.selected_app, APPS_PER_ROW, false);
                state.selected_app = index;
                self.first_visible = visible;
                true
            }
            Actions::Down => {
                handler.animate_placeholder(crate::widgets::app_tile::ANIMATION_TIME);
                let (index, visible) = self.increment_index(state.selected_app, APPS_PER_ROW, true);
                state.selected_app = index;
                self.first_visible = visible;
                true
            }
            Actions::Left => {
                handler.animate_placeholder(crate::widgets::app_tile::ANIMATION_TIME);
                let (index, visible) = self.increment_index(state.selected_app, 1, false);
                state.selected_app = index;
                self.first_visible = visible;
                true
            }
            Actions::Right => { 
                handler.animate_placeholder(crate::widgets::app_tile::ANIMATION_TIME);
                let (index, visible) = self.increment_index(state.selected_app, 1, true);
                state.selected_app = index;
                self.first_visible = visible;
                true 
            }
            Actions::Accept => {
                crate::restrictions::verify_restricted_action(state, |state| {
                    let state = state.downcast_mut::<YaffeState>().unwrap();
                    start_game(state);
                });
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

    fn got_focus(&mut self, _: Rectangle, handler: &mut DeferredAction) {
        handler.animate_placeholder(crate::widgets::app_tile::ANIMATION_TIME);
    }

    fn render(&mut self, state: &YaffeState, rect: Rectangle, delta_time: f32, piet: &mut Graphics2D) {
        self.update(state, &rect, delta_time);

        let plat = state.get_platform();

        //Height needs to be based on image aspect * width
        let focused = state.is_widget_focused(self);
        for i in 0..plat.apps.len() {
            if i == state.selected_app && focused { continue; }

            let tile = &mut self.tiles[i];
            tile.render(&state.settings, focused, &plat.apps[i], piet);
        }

        if let Some(tile) = self.tiles.get_mut(state.selected_app) {
            tile.render(&state.settings, focused, &plat.apps[state.selected_app], piet);
        }
    }
}
impl AppList {
    fn update(&mut self, state: &YaffeState, rect: &Rectangle, delta_time: f32) {
        //Check the length of our cache vs actual in case a game was added
        //to this platform while we were on it
        if self.cached_platform != state.selected_platform ||
            self.tiles.len() != state.get_platform().apps.len() {
            self.tiles.clear();

            for i in 0..state.get_platform().apps.len() {
                self.tiles.push(AppTile::new(self.queue.clone(), i));
            }

            self.cached_platform = state.selected_platform;
        }

        for exe in self.tiles.iter_mut() {
            exe.update(state, delta_time);
        }

        self.update_tiles(state, rect);
    }

    fn update_tiles(&mut self, state: &YaffeState,  rect: &Rectangle) {
        let platform = state.get_platform();

        //Calculate total size for inner list
        let margin_x = rect.width() * MARGIN;
        let margin_y = rect.height() * MARGIN;
        let list_rect = Rectangle::from_tuples((rect.left() + margin_x, rect.top() + margin_y), (rect.right() - margin_x, rect.bottom() - margin_y));

        //Get size each tile should try to stretch to
        let (tiles_x, tiles_y, ideal_tile_size) = self.get_ideal_tile_size(state, platform.kind != crate::platform::PlatformType::Enumlator, &list_rect);
        self.tiles_x = tiles_x;
        self.tiles_y = tiles_y;

        let mut effective_i = 0;
        for exe in self.tiles.iter_mut() {
            exe.apply_filter(&state.search_info, &platform.apps);

            //Size each tile according to its aspect ratio and the ideal size
            AppList::size_individual_tile(state, exe, &ideal_tile_size);

            let x = (effective_i % self.tiles_x) as f32;
            let y = ((effective_i / self.tiles_x) - (self.first_visible / self.tiles_x)) as f32;

            let offset = (ideal_tile_size - exe.size) / 2.0;

            let position = V2::new(ideal_tile_size.x * x + offset.x + list_rect.left(), 
                                   ideal_tile_size.y * y + offset.y + list_rect.top());
            exe.position = position;

            if exe.is_visible() { effective_i += 1; }
        }
    }

    fn get_ideal_tile_size(&self, state: &YaffeState, max: bool, rect: &Rectangle) -> (isize, isize, V2) {
        let mut width = 0.;
        let mut height = 0.;
        let mut tiles_x = 1;
        let mut tiles_y = 1;
        let mut bitmap_size = V2::new(0., 0.);
        if self.tiles.len() > 0 {
            //Get widest boxart image
            let mut max_width = 0.;
            for exe in self.tiles.iter() {
                let size = exe.get_image_size(state);
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
                    height= aspect * width;
                } else {
                    let aspect = bitmap_size.x / bitmap_size.y;
                    height = menu_size.y / state.settings.get_i32(crate::SettingNames::ItemsPerColumn) as f32;
                    width = aspect * height;
                }
                tiles_x = (menu_size.x / width) as isize;
                tiles_y = (menu_size.y / height) as isize;
            }
        }

        (tiles_x, tiles_y, V2::new(width, height))
    }

    fn size_individual_tile(state: &YaffeState, tile: &mut AppTile, size: &V2) {
        let mut tile_size = *size;

        //By default on the recents menu it chooses the widest game boxart (see pFindMax in GetTileSize)
		//We wouldn't want vertical boxart to stretch to the horizontal dimensions
		//This will scale boxart that is different aspect to fit within the tile_size.Height
        let bitmap_size = tile.get_image_size(state);
        let real_aspect = bitmap_size.x / bitmap_size.y;
        let tile_aspect = tile_size.x / tile_size.y;

        //If an aspect is wider than it is tall, it is > 1
		//If the two aspect ratios are on other sides of one, it means we need to scale
		if f32::is_sign_positive(real_aspect - 1.) != f32::is_sign_positive(tile_aspect - 1.) {
			tile_size.x = tile_size.y * real_aspect;
		}

		tile.size = tile_size;
    }

    fn increment_index(&self, index: usize, amount: usize, forward: bool) -> (usize, isize) {
        let mut index = index as isize;
        let old_index = index;
        let one = if forward { 1 } else { -1 };

        //Since certain roms could be filtered out,
	    //we will loop until we have incremented the proper amount of times
        let mut first_visible = self.first_visible;
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
        while index > first_visible + (self.tiles_x * self.tiles_y) - 1 { first_visible += self.tiles_x; }
        assert!(self.first_visible >= 0);
        assert!(index >= 0);

        (index as usize, first_visible)
    }
}

fn start_game(state: &mut YaffeState) {
    if let Some(exe) = state.get_executable() {

        //This should never fail since we got it from the database
        let (path, args, roms) = crate::database::get_platform_info(exe.platform_id).log_message_if_fail("Platform not found");

        let mut process = &mut std::process::Command::new(path);
        if exe.platform_id > 0 {
            crate::database::update_game_last_run(exe).log_if_fail();
            let exe_path = std::path::Path::new(&roms).join(&exe.file);

            process = process.arg(exe_path.to_str().unwrap());
            if !args.is_empty() { process = process.args(args.split(' ')); }
        } else {
            process = process.args(args.split(' '));
        }

        if let Some(process) = process.spawn().display_failure("Unable to start game", state) {
            let mut overlay = state.overlay.borrow_mut();
            overlay.set_process(process);
            //We could refresh so our recent games page updates, but I dont think that's desirable
        }
    }
}