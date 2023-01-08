use crate::{YaffeState, platform::PlatformType, Actions, DeferredAction, widget, LogicalPosition, LogicalSize, ScaleFactor, Rect};
use crate::modals::{PlatformDetailModal, on_update_platform_close};
use crate::ui::{MARGIN, get_font_color, get_font_unfocused_color, get_accent_color, get_accent_unfocused_color, get_font_size, MENU_BACKGROUND, display_modal};

widget!(pub struct PlatformList {});
impl crate::ui::Widget for PlatformList {
    fn action(&mut self, state: &mut YaffeState, action: &Actions, handler: &mut DeferredAction) -> bool {
        match action {
            Actions::Down =>  {
                if state.selected_platform < state.platforms.len() - 1 { 
                    state.selected_platform += 1;
                    handler.load_plugin(crate::plugins::NavigationAction::Initialize);
                }
                true
            }
            Actions::Up => {
                if state.selected_platform > 0 { 
                    state.selected_platform -= 1;
                    handler.load_plugin(crate::plugins::NavigationAction::Initialize);
                }
                true
            }
            Actions::Accept => {
                handler.focus_widget(crate::get_widget_id!(crate::widgets::AppList));
                true
            }
            Actions::Info => {
                let platform = state.get_platform();
                if platform.kind == PlatformType::Emulator {
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
        graphics.draw_text(LogicalPosition::new(rect.width() - title.width().to_logical(graphics) - MARGIN, MARGIN), get_font_color(&state.settings), &title);

        let text_color = if crate::is_widget_focused!(state, PlatformList) { get_font_color(&state.settings) } else { get_font_unfocused_color(&state.settings) };

        let font_size = get_font_size(&state.settings, graphics);

        let selected_index = state.selected_platform;
        let right = rect.right();
        let mut y = 10.;
        let mut plat_kind = -1;
        for (i, p) in state.platforms.iter().enumerate() {
            //Header for the specific platform type
            if p.kind as i32 != plat_kind {
                y = draw_header(graphics, state, y, rect.width(), p.kind);
                plat_kind = p.kind as i32;
            }

            
            //Highlight bar
            let height = font_size.to_logical(graphics);
            if i == selected_index {
                let rect = Rect::from_tuples((rect.left(), y), (right, y + height));

                if crate::is_widget_focused!(state, PlatformList) { graphics.draw_rectangle(rect, get_accent_color(&state.settings)); }
                else { graphics.draw_rectangle(rect, get_accent_unfocused_color(&state.settings)); }
            }
            
            //Label
            let name_label = crate::ui::get_drawable_text(font_size, &p.name);
            graphics.draw_text(LogicalPosition::new(MARGIN, y), text_color, &name_label);
    
            if let PlatformType::Emulator = p.kind {
                //Count
                let num_label = crate::ui::get_drawable_text(font_size, &p.apps.len().to_string());
                graphics.draw_text(LogicalPosition::new(right - num_label.width() - MARGIN, y), text_color, &num_label);
            }
            y += height;
        }
    }
}

fn draw_header(graphics: &mut crate::Graphics, state: &YaffeState, y: f32, width: f32, kind: PlatformType) -> f32 {
    const ICON_SIZE: f32 = 28.;
    let image = match kind {
        PlatformType::Emulator => crate::assets::Images::Emulator,
        PlatformType::Plugin => crate::assets::Images::App,
        PlatformType::Recents => crate::assets::Images::Recent,
    };

    let y = y + MARGIN * 2.;
    let i = crate::assets::request_image(graphics, image).unwrap();
    i.render(graphics, Rect::point_and_size(LogicalPosition::new(MARGIN, y), LogicalSize::new(ICON_SIZE, ICON_SIZE)));
    
    let y = y + ICON_SIZE;
    graphics.draw_line(LogicalPosition::new(ICON_SIZE + MARGIN * 2., y), LogicalPosition::new(width - MARGIN, y), 2., get_font_color(&state.settings));
    
    y + MARGIN
}