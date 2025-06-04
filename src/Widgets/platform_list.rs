use crate::{YaffeState, state::GroupType, Actions, DeferredAction, widget, LogicalPosition, LogicalSize, ScaleFactor, Rect};
use crate::modals::{PlatformDetailModal, on_update_platform_close};
use crate::ui::{MARGIN, MENU_BACKGROUND, display_modal};

widget!(pub struct PlatformList {});
impl crate::ui::Widget for PlatformList {
    fn action(&mut self, state: &mut YaffeState, action: &Actions, handler: &mut DeferredAction) -> bool {
        match action {
            Actions::Down =>  {
                if state.selected_platform < state.platforms.len() - 1 { 
                    state.selected_platform += 1;
                    state.selected_app = 0;
                    handler.load_plugin(crate::plugins::NavigationAction::Load);
                }
                true
            }
            Actions::Up => {
                if state.selected_platform > 0 { 
                    state.selected_platform -= 1;
                    state.selected_app = 0;
                    handler.load_plugin(crate::plugins::NavigationAction::Load);
                }
                true
            }
            Actions::Accept => {
                handler.focus_widget(crate::get_widget_id!(crate::widgets::AppList));
                true
            }
            Actions::Info => {
                let platform = state.get_platform();
                if let GroupType::Emulator = platform.kind {
                    let modal = Box::new(PlatformDetailModal::from_existing(platform));
                    display_modal(state, "Platform Info", Some("Save"), modal, Some(on_update_platform_close));
                }
                true
            }
            _ => false
        }
    }

    fn render(&mut self, graphics: &mut crate::Graphics, state: &YaffeState) {
        //Background
        let rect = graphics.bounds;
        graphics.draw_rectangle(rect, MENU_BACKGROUND);

        //Title
        let title = crate::ui::get_drawable_text(32. * graphics.scale_factor, "Yaffe");
        graphics.draw_text(LogicalPosition::new(rect.width() - title.width().to_logical(graphics) - MARGIN, MARGIN), graphics.font_color(), &title);

        let text_color = if crate::is_focused!(state) { graphics.font_color() } else { graphics.font_unfocused_color() };

        let font_size = graphics.font_size();

        let selected_index = state.selected_platform;
        let right = rect.right();
        let mut y = 10.;
        let mut plat_kind = -1;
        for (i, p) in state.platforms.iter().enumerate() {
            //Header for the specific platform type
            if p.kind as i32 != plat_kind {
                y = draw_header(graphics, y, rect.width(), p.kind);
                plat_kind = p.kind as i32;
            }
            
            let name_label = crate::ui::get_drawable_text_with_wrap(font_size, &p.name, rect.width() - font_size * 2.);
            
            //Highlight bar
            let height = name_label.height();
            if i == selected_index {
                let rect = Rect::from_tuples((rect.left(), y), (right, y + height));

                if crate::is_focused!(state) { graphics.draw_rectangle(rect, graphics.accent_color()); }
                else { graphics.draw_rectangle(rect, graphics.accent_unfocused_color()); }
            }
            
            //Label
            graphics.draw_text(LogicalPosition::new(MARGIN, y), text_color, &name_label);
    
            if let GroupType::Emulator = p.kind {
                //Count
                let num_label = crate::ui::get_drawable_text(font_size, &p.apps.len().to_string());
                graphics.draw_text(LogicalPosition::new(right - num_label.width() - MARGIN, y), text_color, &num_label);
            }
            y += height;
        }
    }
}

fn draw_header(graphics: &mut crate::Graphics, y: f32, width: f32, kind: GroupType) -> f32 {
    const ICON_SIZE: f32 = 28.;
    let image = match kind {
        GroupType::Emulator => crate::assets::Images::Emulator,
        GroupType::Plugin => crate::assets::Images::App,
        GroupType::Recents => crate::assets::Images::Recent,
    };

    let y = y + MARGIN * 2.;
    let i = crate::assets::request_image(graphics, image).unwrap();
    i.render(graphics, Rect::point_and_size(LogicalPosition::new(MARGIN, y), LogicalSize::new(ICON_SIZE, ICON_SIZE)));
    
    let y = y + ICON_SIZE;
    graphics.draw_line(LogicalPosition::new(ICON_SIZE + MARGIN * 2., y), LogicalPosition::new(width - MARGIN, y), 2., graphics.font_color());
    
    y + MARGIN
}