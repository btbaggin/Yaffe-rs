use crate::controls::MENU_BACKGROUND;
use crate::state::MetadataSearch;
use crate::ui::{AnimationManager, LayoutElement, RevertFocusAction, UiElement, WidgetId, MARGIN};
use crate::{widget, Actions, DeferredAction, LogicalPosition, LogicalSize, Rect, ScaleFactor, YaffeState};

const MIN_NAME_WIDTH: f32 = 175.;

widget!(
    pub struct SearchBar {
        active_search: usize = 0,
        highlight_offset: f32 = 0.,
        cached_platform: i64 = -1,
        searches: Vec<MetadataSearch> = vec!(),
        offset: LogicalPosition = LogicalPosition::new(0., -1.),
        name_width: f32 = 0.
    }
);
impl UiElement<YaffeState> for SearchBar {
    fn action(
        &mut self,
        state: &mut YaffeState,
        animations: &mut AnimationManager,
        action: &Actions,
        handler: &mut DeferredAction<YaffeState>,
    ) -> bool {
        match action {
            Actions::Back => {
                handler.defer(RevertFocusAction);
                state.filter = None;
                true
            }
            Actions::Accept => {
                handler.focus_widget(crate::APP_LIST_ID);
                let filter = self.searches[self.active_search].clone();

                //If our current item is no longer visible because it was filtered out
                //Find the first visible item so it can be selected in the app list
                if let Some(tile) = state.get_selected_tile() {
                    if !filter.item_is_visible(tile) {
                        state.selected.tile_index = 0;

                        while let Some(tile) = state.get_selected_tile() {
                            if filter.item_is_visible(tile) {
                                break;
                            }
                            state.selected.tile_index += 1;
                        }
                    }
                }
                state.filter = Some(filter);
                true
            }
            Actions::Left => {
                self.switch_option(state, -1, animations);
                true
            }
            Actions::Right => {
                self.switch_option(state, 1, animations);
                true
            }
            Actions::Down => {
                if self.has_less_search_options() {
                    self.switch_search(state, -1);
                    return true;
                }
                false
            }
            Actions::Up => {
                if self.has_more_search_options() {
                    self.switch_search(state, 1);
                    return true;
                }
                false
            }
            _ => false,
        }
    }

    fn got_focus(&mut self, state: &YaffeState, animations: &mut AnimationManager) {
        animations
            .animate(self, crate::offset_of!(SearchBar => offset: LogicalPosition => y), 0.)
            .duration(0.2)
            .start();

        // Make sure we always have the name search since thats default
        let group = state.get_selected_group();
        if group.id != self.cached_platform {
            self.searches.clear();
            self.searches.extend(group.search.clone());

            self.active_search = 0;
            self.highlight_offset = 0.;

            let current = &mut self.searches[self.active_search];
            current.set_mask(&group.tiles);
            current.selected = None;

            self.cached_platform = group.id;
        }
    }

    fn lost_focus(&mut self, state: &YaffeState, animations: &mut AnimationManager) {
        if state.filter.is_none() {
            animations
                .animate(self, crate::offset_of!(SearchBar => offset: LogicalPosition => y), -1.)
                .duration(0.2)
                .start();
        }
    }

    fn render(&mut self, graphics: &mut crate::Graphics, _: &YaffeState, current_focus: &WidgetId) {
        let Some(current_search) = &self.searches.get(self.active_search) else {
            return;
        };

        let rect = self.layout();
        let rect = Rect::point_and_size(*rect.top_left() + (self.offset * rect.height()), rect.size());

        let font_size = graphics.font_size();
        let name_label = crate::ui::get_drawable_text(graphics, font_size, &current_search.name);
        self.name_width = f32::max(MIN_NAME_WIDTH, name_label.width() + MARGIN * 2.);

        let filter_start = rect.left() + self.name_width;
        let item_size = (rect.right() - filter_start) / current_search.options.len() as f32;

        graphics.draw_rectangle(rect, MENU_BACKGROUND);
        let focused_color = if current_focus == &crate::SEARCH_BAR_ID {
            graphics.font_color()
        } else {
            graphics.font_unfocused_color()
        };

        //Filter option name
        let filter_rect =
            Rect::new(*rect.top_left(), LogicalSize::new(rect.left() + self.name_width, rect.top() + rect.height()));

        //Highlight
        let mut highlight_position = rect.left() + self.highlight_offset;
        let mut highlight_width = self.name_width;
        if current_search.selected.is_some() {
            highlight_position += self.name_width;
            highlight_width = item_size;
        }

        let r =
            Rect::from_tuples((highlight_position, rect.top()), (highlight_position + highlight_width, rect.bottom()));
        graphics.draw_rectangle(r, graphics.accent_color());

        let mid = filter_rect.left() + filter_rect.width() / 2.;
        let half = name_label.width().to_logical(graphics) / 2.;
        graphics.draw_text(
            LogicalPosition::new(
                mid - half,
                (filter_rect.top() + filter_rect.height() / 2.) - name_label.height().to_logical(graphics) / 2.,
            ),
            focused_color,
            &name_label,
        );

        const ARROW_SIZE: f32 = 10.;
        const ARROW_HEIGHT: f32 = 5.;
        if self.has_less_search_options() {
            //Down arrow
            graphics.draw_line(
                LogicalPosition::new(mid - ARROW_SIZE, filter_rect.bottom() - 7. - ARROW_HEIGHT),
                LogicalPosition::new(mid, filter_rect.bottom() - 7.),
                2.,
                focused_color,
            );
            graphics.draw_line(
                LogicalPosition::new(mid, filter_rect.bottom() - 7.),
                LogicalPosition::new(mid + ARROW_SIZE, filter_rect.bottom() - 7. - ARROW_HEIGHT),
                2.,
                focused_color,
            );
        }
        if self.has_more_search_options() {
            //Up arrow
            graphics.draw_line(
                LogicalPosition::new(mid - ARROW_SIZE, filter_rect.top() + 12.),
                LogicalPosition::new(mid, filter_rect.top() + 7.),
                2.,
                focused_color,
            );
            graphics.draw_line(
                LogicalPosition::new(mid, filter_rect.top() + 7.),
                LogicalPosition::new(mid + ARROW_SIZE, filter_rect.top() + 12.),
                2.,
                focused_color,
            );
        }

        let mask = current_search.mask;
        for (i, o) in current_search.options.iter().enumerate() {
            let item_start = filter_start + (i as f32 * item_size);

            //Filter item
            //If there are no items that match a certain filter we will draw it unfocused
            let color = if mask & 1 << i != 0 { focused_color } else { graphics.font_unfocused_color() };
            let item_label = crate::ui::get_drawable_text(graphics, font_size, o);

            let label_half = LogicalSize::new(
                item_label.width().to_logical(graphics) / 2.,
                item_label.height().to_logical(graphics) / 2.,
            );
            graphics.draw_text(
                LogicalPosition::new(item_start + item_size / 2. - label_half.x, rect.top() + label_half.y),
                color,
                &item_label,
            );
        }
    }
}
impl SearchBar {
    fn has_more_search_options(&self) -> bool { self.active_search < self.searches.len() - 1 }

    fn has_less_search_options(&self) -> bool { self.active_search > 0 }

    fn switch_search(&mut self, state: &YaffeState, increment: isize) {
        let group = state.get_selected_group();
        self.active_search = (self.active_search as isize + increment) as usize;
        self.highlight_offset = 0.;
        self.searches[self.active_search].selected = None;
        self.searches[self.active_search].set_mask(&group.tiles);
    }

    fn switch_option(&mut self, state: &mut YaffeState, increment: isize, animations: &mut AnimationManager) {
        let filter_start = self.position.x + self.name_width;
        let item_size =
            (self.position.x + self.size.x - filter_start) / self.searches[self.active_search].options.len() as f32;
        self.searches[self.active_search].increment_index(increment);

        animations
            .animate(
                self,
                crate::offset_of!(SearchBar => highlight_offset),
                self.searches[self.active_search].selected.unwrap_or(0) as f32 * item_size,
            )
            .duration(0.1)
            .start();
        state.filter = Some(self.searches[self.active_search].clone());
    }
}
