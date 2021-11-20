use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::{YaffeState, platform::PlatformType, Actions, DeferredAction, widget, LogicalPosition, LogicalSize, Rect};
use crate::{colors::*, ui::*, font::*};
use crate::modals::{PlatformDetailModal, SettingsModal, on_update_platform_close, on_settings_close, ModalSize, display_modal};

widget!(pub struct PlatformList {});
impl super::Widget for PlatformList {
    fn action(&mut self, state: &mut YaffeState, action: &Actions, handler: &mut DeferredAction) -> bool {
        match action {
            Actions::Down =>  {
                if state.selected_platform < state.platforms.len() - 1 { 
                    state.selected_platform += 1;
                    handler.load_plugin(crate::plugins::PluginLoadType::Initial);
                }
                true
            }
            Actions::Up => {
                if state.selected_platform > 0 { 
                    state.selected_platform -= 1;
                    handler.load_plugin(crate::plugins::PluginLoadType::Initial);
                }
                true
            }
            Actions::Accept => {
                handler.focus_widget(crate::get_widget_id!(crate::widgets::AppList));
                true
            }
            Actions::Info => {
                let platform = state.get_platform();
                match platform.kind {
                    PlatformType::Emulator => {
                        let modal = Box::new(PlatformDetailModal::from_existing(platform, platform.id.unwrap()));
                        display_modal(state, "Platform Info", Some("Save"), modal, ModalSize::Half, Some(on_update_platform_close));
                    },
                    PlatformType::Plugin => {
                        let (plugin, _) = platform.get_plugin(state).unwrap();
                        let plugin = plugin.borrow().file.clone();
                        let modal = Box::new(SettingsModal::new(&state.settings, Some(&plugin)));
                        
                        display_modal(state, "Settings", Some("Save"), modal, ModalSize::Half, Some(on_settings_close));
                    },
                    _ => {},
                }
                true
            }
            _ => false
        }
    }

    fn render(&mut self, state: &YaffeState, rect: Rect, _: f32, piet: &mut Graphics2D) {
        //Background
        piet.draw_rectangle(rect.into(), MENU_BACKGROUND);

        //Title
        let title = crate::widgets::get_drawable_text(get_title_font_size(state), "Yaffe");
        piet.draw_text(LogicalPosition::new(rect.width() - title.width() - 30., MARGIN), get_font_color(&state.settings), &title);

        let text_color = if state.is_widget_focused(self) { get_font_color(&state.settings) } else { get_font_unfocused_color(&state.settings) };

        let selected_index = state.selected_platform;
        let right = rect.right();
        let mut y = 10.;
        let mut plat_kind = -1;
        for (i, p) in state.platforms.iter().enumerate() {
            //Header for the specific platform type
            if p.kind as i32 != plat_kind {
                y = draw_header(piet, state, y, rect.width(), p.kind, 28.);
                plat_kind = p.kind as i32;
            }

            let name_label = super::get_drawable_text(FONT_SIZE, &p.name);
            
            //Highlight bar
            let height = name_label.height();
            if i == selected_index {
                let rect = Rectangle::from_tuples((rect.left(), y), (right, y + height));

                if state.is_widget_focused(self) { piet.draw_rectangle(rect, get_accent_color(&state.settings)); }
                else { piet.draw_rectangle(rect, get_accent_unfocused_color(&state.settings)); }
            }
            
            //Label
            piet.draw_text(LogicalPosition::new(MARGIN, y), text_color, &name_label);
    
            if let PlatformType::Emulator = p.kind {
                //Count
                let num_label = super::get_drawable_text(FONT_SIZE, &p.apps.len().to_string());
                piet.draw_text(LogicalPosition::new(right - num_label.width() - MARGIN, y), text_color, &num_label);
            }
            y += height;
        }
    }
}

fn draw_header(piet: &mut Graphics2D, state: &YaffeState, y: f32, width: f32, kind: PlatformType, icon_size: f32) -> f32 {
    let image = match kind {
        PlatformType::Emulator => crate::assets::Images::Emulator,
        PlatformType::Plugin => crate::assets::Images::App,
        PlatformType::Recents => crate::assets::Images::Recent,
    };

    let y = y + MARGIN * 2.;
    let i = crate::assets::request_preloaded_image(piet, image);
    i.render(piet, Rect::point_and_size(LogicalPosition::new(MARGIN, y), LogicalSize::new(icon_size, icon_size)).into());
    
    let y = y + icon_size;
    piet.draw_line(LogicalPosition::new(icon_size + MARGIN * 2., y), LogicalPosition::new(width - MARGIN, y), 2., get_font_color(&state.settings));
    
    y + MARGIN
}