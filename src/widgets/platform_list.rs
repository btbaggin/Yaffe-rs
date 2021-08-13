use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::{YaffeState, Actions, DeferredAction, widget, V2, Rect};
use crate::{colors::*, ui::*, font::*};
use crate::modals::{PlatformDetailModal, ModalSize, display_modal};

widget!(pub struct PlatformList {});
impl super::Widget for PlatformList {
    fn action(&mut self, state: &mut YaffeState, action: &Actions, handler: &mut DeferredAction) -> bool {
        match action {
            Actions::Down =>  {
                if state.selected_platform < state.platforms.len() - 1 { 
                    state.selected_platform += 1;
                }
                true
            }
            Actions::Up => {
                if state.selected_platform > 0 { 
                    state.selected_platform -= 1;
                }
                true
            }
            Actions::Accept => {
                handler.focus_widget(crate::get_widget_id!(crate::widgets::AppList));
                true
            }
            Actions::Info => {
                let platform = state.get_platform();
                if let crate::platform::PlatformType::App = platform.kind {
                    let modal = Box::new(PlatformDetailModal::from_existing(platform));
                    display_modal(state, "Platform Info", Some("Save"), modal, ModalSize::Half, Some(crate::modals::on_update_application_close));
                }
                true
            }
            _ => false
        }
    }

    fn render(&mut self, state: &YaffeState, rect: Rectangle, _: f32, piet: &mut Graphics2D) {
        //Background
        piet.draw_rectangle(rect.clone(), MENU_BACKGROUND);

        //Title
        let title = crate::widgets::get_drawable_text(get_title_font_size(state), "Yaffe");
        piet.draw_text(V2::new(rect.width() - title.width() - 30., MARGIN), get_font_color(&state.settings), &title);

        let text_color = if state.is_widget_focused(self) { get_font_color(&state.settings) } else { get_font_unfocused_color(&state.settings) };

        let selected_index = state.selected_platform;
        let right = rect.right();
        let mut y = 10.;
        let plat_kind = -1;
        for (i, p) in state.platforms.iter().enumerate() {
            //Header for the specific platform type
            if p.kind as i32 != plat_kind {
                y = draw_header(piet, state, y, rect.width(), p.kind, 28.);
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
            piet.draw_text(V2::new(crate::ui::MARGIN, y), text_color, &name_label);
    
            //Count
            let num_label = super::get_drawable_text(FONT_SIZE, &p.apps.len().to_string());
            piet.draw_text(V2::new(right - num_label.width() - MARGIN, y), text_color, &num_label);
            y += height;
        }
    }
}

fn draw_header(piet: &mut Graphics2D, state: &YaffeState, y: f32, width: f32, kind: crate::platform::PlatformType, icon_size: f32) -> f32 {
    let image = match kind {
        crate::platform::PlatformType::Enumlator => crate::assets::Images::Emulator,
        crate::platform::PlatformType::App => crate::assets::Images::App,
        crate::platform::PlatformType::Recents => crate::assets::Images::Recent,
    };

    let y = y + MARGIN * 2.;
    let i = crate::assets::request_preloaded_image(piet, image);
    i.render(piet, Rect::point_and_size(V2::new(MARGIN, y), V2::new(icon_size, icon_size)));
    
    let y = y + icon_size;
    piet.draw_line(V2::new(icon_size + MARGIN * 2., y), V2::new(width - MARGIN, y), 2., get_font_color(&state.settings));
    
    y + MARGIN
}