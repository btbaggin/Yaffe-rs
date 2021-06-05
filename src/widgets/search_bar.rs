use druid_shell::kurbo::{Rect, Point, Size, Line};
use druid_shell::piet::{RenderContext, Piet, TextLayout};
use crate::{YaffeState, create_widget, Actions, DeferredAction};
use crate::colors::*;

const SEARCH_OPTION_NONE: i32 = 0;
const SEARCH_OPTION_NAME: i32 = 1;
const SEARCH_OPTION_PLAYERS: i32 = 2;
const SEARCH_OPTION_MAX: i32 = SEARCH_OPTION_PLAYERS;

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
            SEARCH_OPTION_PLAYERS => { exe.players == self.start - self.index as u8 }
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
            else if i == (self.end - self.start) as isize { self.index = -1; return; }
        }
    }
}

create_widget!(SearchBar, active: bool = false);
impl super::Widget for SearchBar {
    fn layout(&self, space: &Rect, size: Size) -> Rect {
        let position = Point::new(space.x0, space.y0 - size.height);
        Rect::from((position, size))
    }

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
                search.increment_index(mask, -1);
                true
            }
            Actions::Right => {
                search.increment_index(mask, 1);
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

    fn got_focus(&mut self, layout: &Rect, handle: &mut DeferredAction) {
        handle.animate(self, Point::new(layout.x0, layout.y0 + layout.height()), 0.2);
    }

    fn lost_focus(&mut self, layout: &Rect, handle: &mut DeferredAction) {
        if !self.active {
            handle.animate(self, Point::new(layout.x0, layout.y0), 0.2);
        }
    }

    fn render(&mut self, state: &YaffeState, rect: Rect, piet: &mut Piet) {
        const NAME_WIDTH: f64 = 175.;

        let search = &state.search_info;
        let filter_start = rect.x0 + NAME_WIDTH;
        let start = search.start;
        let end = search.end;
        let name = match search.option {
            SEARCH_OPTION_NONE => "None",
            SEARCH_OPTION_NAME => "Name",
            SEARCH_OPTION_PLAYERS => "Players",
            _ => panic!("Unknown filter option"),
        };
        
        let item_size = (rect.x1 - filter_start) / (end - start + 1) as f64;

        piet.fill(rect, &MENU_BACKGROUND);
        let focused_color = if state.is_widget_focused(self) { get_font_color(&state.settings) } else { get_font_unfocused_color(&state.settings) };

        //Filter option name
        let filter_rect = Rect::from((Point::new(rect.x0, rect.y0), Size::new(NAME_WIDTH, rect.height())));
        if search.index < 0 {
            piet.fill(filter_rect, get_accent_color(&state.settings));
        }

        let name_label = super::get_drawable_text(piet, crate::font::FONT_SIZE, &name, focused_color.clone());
        let name_half = name_label.size() / 4.;
        piet.draw_text(&name_label, (rect.x0 + crate::ui::MARGIN, rect.y0 + name_half.height));

        const ARROW_SIZE: f64 = 10.;
        const ARROW_HEIGHT: f64 = 5.;
        let mid = filter_rect.x0 + filter_rect.width() / 2.;
        if search.option > SEARCH_OPTION_NONE { 
            //Down arrow
            piet.stroke(Line::new((mid - ARROW_SIZE, filter_rect.y1 - 7. - ARROW_HEIGHT), (mid, filter_rect.y1 - 7.)), &focused_color, 2.); 
            piet.stroke(Line::new((mid, filter_rect.y1 - 7.), (mid + ARROW_SIZE, filter_rect.y1 - 7. - ARROW_HEIGHT)), &focused_color, 2.);
        }
         if search.option < SEARCH_OPTION_MAX { 
            //Up arrow
            piet.stroke(Line::new((mid - ARROW_SIZE, filter_rect.y0 + 12.), (mid, filter_rect.y0 + 7.)), &focused_color, 2.); 
            piet.stroke(Line::new((mid, filter_rect.y0 + 7.), (mid + ARROW_SIZE, filter_rect.y0 + 12.)), &focused_color, 2.); 
          }

        let mask = get_exists_mask(search.option, &state.get_platform().apps);
        for i in start..=end {
            let item_start = filter_start + ((i - start) as f64 * item_size);

            //Heighlight
            if search.index + start as isize == i as isize {
                let r = Rect::new(item_start, rect.y0, item_start + item_size, rect.y1);
                piet.fill(r, get_accent_color(&state.settings));
            }

            //Filter item
            //If there are no items that match a certain filter we will draw it unfocused
            let bit = i - start;
            let color = if mask & 1 << bit != 0 { focused_color.clone() } else { get_font_unfocused_color(&state.settings) };
            let item_label = super::get_drawable_text(piet, crate::font::FONT_SIZE, &String::from(i as char), color);
            
            let label_half = item_label.size() / 2.;
            piet.draw_text(&item_label, (item_start + item_size / 2. - label_half.width, rect.y0  + label_half.height));
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