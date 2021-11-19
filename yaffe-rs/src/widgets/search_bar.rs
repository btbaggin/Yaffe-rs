use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::{YaffeState, widget, Actions, DeferredAction, LogicalPosition, LogicalSize};
use crate::colors::*;
use crate::Rect;
const SEARCH_OPTION_NONE: i32 = 0;
const SEARCH_OPTION_NAME: i32 = 1;
const SEARCH_OPTION_PLAYERS: i32 = 2;
const SEARCH_OPTION_MAX: i32 = SEARCH_OPTION_PLAYERS;
const NAME_WIDTH: f32 = 175.;

pub struct SearchInfo {
    option: i32,
    index: isize,
    start: u8,
    end: u8,
}
impl SearchInfo {
    pub fn new() -> SearchInfo {
        SearchInfo { option: SEARCH_OPTION_NONE, index: -1, start: 0, end: 0 }
    }

    pub fn item_is_visible(&self, exe: &crate::Executable) -> bool {
        if self.index == -1 { return true; }

        match self.option {
            SEARCH_OPTION_NONE => { true }
            SEARCH_OPTION_NAME => { 
                let c = (self.start + self.index as u8) as char;
                exe.name.starts_with(c) 
            }
            SEARCH_OPTION_PLAYERS => { exe.players == (self.index + 1) as u8 }
            _ => panic!("Unknown filter option"),
        }
    }

    fn increment_index(&mut self, mask: u64, amount: isize) {
        let mut i = self.index;
        //self.index must be assigned in all paths of this loop
        //this loop is guaranteed to end because either the index will hit -1 or self.end
        loop {
            i += amount;
            if i <= -1 { self.index = -1; return; }
            else if mask & 1 << i != 0 { self.index = i; return; }
            else if i >= (self.end - self.start) as isize { self.index = -1; return; }
        }
    }
}

widget!(pub struct SearchBar { 
    active: bool = false,
    highlight_offset: f32 = 0.
});
impl super::Widget for SearchBar {
    fn offset(&self) -> LogicalPosition { LogicalPosition::new(0., -1.) }

    fn action(&mut self, state: &mut YaffeState, action: &Actions, handler: &mut DeferredAction) -> bool {
        let mask = get_exists_mask(state.search_info.option, &state.get_platform().apps);
        let search = &mut state.search_info;
        match action {
            Actions::Back => {
                handler.revert_focus();
                self.active = false;
                true
            }
            Actions::Accept => {
                handler.focus_widget(crate::get_widget_id!(crate::widgets::AppList));
                self.active = true;

                //If our current item is no longer visible because it was filtered out
                //Find the first visible item so it can be selected in the app list
                if let Some(exe) = state.get_executable() {
                    if !state.search_info.item_is_visible(exe) {  
                        state.selected_app = 0;

                        while let Some(exe) = state.get_executable() {
                            if state.search_info.item_is_visible(exe) { break; }
                            state.selected_app += 1;  
                        }
                    }
                }
                true
            }
            Actions::Left => {
                let filter_start = self.position.x + NAME_WIDTH;
                let item_size = (self.position.x + self.size.x - filter_start) / (search.end - search.start + 1) as f32;
                search.increment_index(mask, -1);

                let offset = crate::offset_of!(SearchBar => highlight_offset);
                handler.animate_f32(self, offset, f32::max(0., search.index as f32 * item_size), 0.1);
                true
            }
            Actions::Right => {
                let filter_start = self.position.x + NAME_WIDTH;
                let item_size = (self.position.x + self.size.x - filter_start) / (search.end - search.start + 1) as f32;
                search.increment_index(mask, 1);

                let offset = crate::offset_of!(SearchBar => highlight_offset);
                handler.animate_f32(self, offset, f32::max(0., search.index as f32 * item_size), 0.1);

                true
            }
            Actions::Down => {
                if search.index == -1 && search.option > SEARCH_OPTION_NONE {
                    search.option -= 1;
                    set_search_base(search);
                    return true;
                }
                false
            }
            Actions::Up => {
                if search.index == -1 && search.option < SEARCH_OPTION_MAX {
                    search.option += 1;
                    set_search_base(search);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn got_focus(&mut self, original: Rectangle, handle: &mut DeferredAction) {
        let offset = crate::offset_of!(SearchBar => position: LogicalPosition => y);
        handle.animate_f32(self, offset, original.bottom(), 0.2);

        self.highlight_offset = 0.;
    }

    fn lost_focus(&mut self, original: Rectangle, handle: &mut DeferredAction) {
        if !self.active {
            let offset = crate::offset_of!(SearchBar => position: LogicalPosition => y);
            handle.animate_f32(self, offset, original.top(), 0.2);
        }
    }

    fn render(&mut self, state: &YaffeState, rect: Rectangle, _: f32, piet: &mut Graphics2D) {

        let search = &state.search_info;
        let filter_start = rect.left() + NAME_WIDTH;
        let start = search.start;
        let end = search.end;
        let name = match search.option {
            SEARCH_OPTION_NONE => "None",
            SEARCH_OPTION_NAME => "Name",
            SEARCH_OPTION_PLAYERS => "Players",
            _ => panic!("Unknown filter option"),
        };
        
        let item_size = (rect.right() - filter_start) / (end - start + 1) as f32;


        piet.draw_rectangle(rect.clone(), MENU_BACKGROUND);
        let focused_color = if state.is_widget_focused(self) { get_font_color(&state.settings) } else { get_font_unfocused_color(&state.settings) };

        //Filter option name
        let filter_rect = Rectangle::new(*rect.top_left(), LogicalSize::new(rect.left() + NAME_WIDTH, rect.top() + rect.height()).into());

        //Highlight
        let mut highlight_position = rect.left() + self.highlight_offset;
        let mut highlight_width = NAME_WIDTH;
        if search.index >= 0 { 
            highlight_position += NAME_WIDTH; 
            highlight_width = item_size;
        }

        let r = Rectangle::from_tuples((highlight_position, rect.top()), (highlight_position + highlight_width, rect.bottom()));
        piet.draw_rectangle(r, get_accent_color(&state.settings));

        let mid = filter_rect.left() + filter_rect.width() / 2.;

        let name_label = super::get_drawable_text(crate::font::FONT_SIZE, &name);
        let half = name_label.width() / 2.;
        piet.draw_text(LogicalPosition::new(mid - half, (filter_rect.top() + filter_rect.height() / 2.) - crate::font::FONT_SIZE / 2.), focused_color, &name_label);

        const ARROW_SIZE: f32 = 10.;
        const ARROW_HEIGHT: f32 = 5.;
        if search.option > SEARCH_OPTION_NONE { 
            //Down arrow
            piet.draw_line(LogicalPosition::new(mid - ARROW_SIZE, filter_rect.bottom() - 7. - ARROW_HEIGHT), LogicalPosition::new(mid, filter_rect.bottom() - 7.), 2., focused_color); 
            piet.draw_line(LogicalPosition::new(mid, filter_rect.bottom() - 7.), LogicalPosition::new(mid + ARROW_SIZE, filter_rect.bottom() - 7. - ARROW_HEIGHT), 2., focused_color);
        }
        if search.option < SEARCH_OPTION_MAX { 
            //Up arrow
            piet.draw_line(LogicalPosition::new(mid - ARROW_SIZE, filter_rect.top() + 12.), LogicalPosition::new(mid, filter_rect.top() + 7.), 2., focused_color); 
            piet.draw_line(LogicalPosition::new(mid, filter_rect.top() + 7.), LogicalPosition::new(mid + ARROW_SIZE, filter_rect.top() + 12.), 2., focused_color); 
        }

        let mask = get_exists_mask(search.option, &state.get_platform().apps);
        for i in start..=end {
            let item_start = filter_start + ((i - start) as f32 * item_size);

            //Filter item
            //If there are no items that match a certain filter we will draw it unfocused
            let bit = i - start;
            let color = if mask & 1 << bit != 0 { focused_color.clone() } else { get_font_unfocused_color(&state.settings) };
            let item_label = super::get_drawable_text(crate::font::FONT_SIZE, &String::from(i as char));
            
            let label_half = LogicalSize::new(item_label.width() / 2., item_label.height() / 2.);
            piet.draw_text(LogicalPosition::new(item_start + item_size / 2. - label_half.x, rect.top()  + label_half.y), color, &item_label);
         }
    }
}

/// Gets a bitmask of filter options that have items we can filter
fn get_exists_mask(option: i32, apps: &Vec<crate::Executable>) -> u64 {
    let mut mask = 0u64;
    match option {
        SEARCH_OPTION_NONE => { /*do nothing, everything is valid*/ }
        SEARCH_OPTION_NAME => {
            for exe in apps.iter() {
                if let Some(c) = exe.name.chars().next() {
                    let c = c.to_uppercase().next().unwrap();
                    mask = mask | (1 << (c as u8 - 'A' as u8));
                }
            }
        }
        SEARCH_OPTION_PLAYERS => {
            for exe in apps.iter() {
                if exe.players > 0 {
                    mask = mask | (1 << exe.players - 1);
                }
            }
        }
        _ => panic!("Unknown filter option"),
    }

    mask
}

fn set_search_base(filter: &mut SearchInfo) {
    match filter.option {
        SEARCH_OPTION_NONE =>  { filter.start = 0; filter.end = 0; },
        SEARCH_OPTION_NAME => { filter.start = 'A' as u8; filter.end = 'Z' as u8 },
        SEARCH_OPTION_PLAYERS => { filter.start = '1' as u8; filter.end = '4' as u8 },
        _ => panic!("Unable filter option"),
    }
}