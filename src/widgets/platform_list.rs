use druid_shell::kurbo::{Rect, Line, Point, Size};
use druid_shell::piet::{RenderContext, Piet, TextLayout};
use crate::{YaffeState, Actions, DeferredAction, create_widget};
use crate::{colors::*, ui::*, font::*};
use crate::modals::{PlatformDetailModal, ModalSize, display_modal};

create_widget!(PlatformList, );
impl super::Widget for PlatformList {
    fn action(&mut self, state: &mut YaffeState, action: &Actions, handler: &mut DeferredAction) -> bool {
        match action {
            Actions::Down =>  {
                if state.selected_platform < state.platforms.len() - 1 { state.selected_platform += 1 }
                true
            }
            Actions::Up => {
                if state.selected_platform > 0 { state.selected_platform -= 1}
                true
            }
            Actions::Accept => {
                handler.focus_widget(crate::get_widget_id!(crate::widgets::AppList));
                true
            }
            Actions::Info => {
                let modal = Box::new(PlatformDetailModal::from_existing(state.get_platform()));
                display_modal(state, "Platform Info", Some("Save"), modal, ModalSize::Half, Some(crate::modals::on_update_application_close));
                true
            }
            _ => false
        }
    }

    fn render(&mut self, state: &YaffeState, rect: Rect, piet: &mut Piet) {
        //Background
        piet.fill(rect, &MENU_BACKGROUND);
        piet.stroke(Line::new((rect.x1, rect.y0), (rect.x1, rect.y1)), &MODAL_BACKGROUND, 1.0);

        //Title
        let title = crate::widgets::get_drawable_text(piet, get_title_font_size(state), "Yaffe", get_font_color(&state.settings));
        piet.draw_text(&title, Point::new(rect.width() - title.size().width - 30., MARGIN));

        let text_color = if state.is_widget_focused(self) { get_font_color(&state.settings) } else { get_font_unfocused_color(&state.settings) };

        let selected_index = state.selected_platform;
        let right = rect.max_x();
        let mut y = 10.;
        let plat_kind = -1;
        for (i, p) in state.platforms.iter().enumerate() {
            //Header for the specific platform type
            if p.kind as i32 != plat_kind {
                y = draw_header(piet, state, y, rect.width(), p.kind, 28.);
            }

            let name_label = super::get_drawable_text(piet, FONT_SIZE, &p.name, text_color.clone());
            
            //Highlight bar
            let height = name_label.size().height;
            if i == selected_index {
                let rect = Rect::new(rect.x0, y, right, y + height);
                if state.is_widget_focused(self) { piet.fill(rect, get_accent_color(&state.settings)); }
                else { piet.fill(rect, &get_accent_unfocused_color(&state.settings)); }
            }
            
            //Label
            piet.draw_text(&name_label, (crate::ui::MARGIN, y));
    
            //Count
            let num_label = super::get_drawable_text(piet, FONT_SIZE, &p.apps.len().to_string(), text_color.clone());
            piet.draw_text(&num_label, (right - num_label.size().width - MARGIN, y));
            y += height;
        }
    }
}

fn draw_header(piet: &mut Piet, state: &YaffeState, y: f64, width: f64, kind: crate::platform::PlatformType, icon_size: f64) -> f64 {
    let image = match kind {
        crate::platform::PlatformType::Enumlator => crate::assets::Images::Emulator,
        crate::platform::PlatformType::App => crate::assets::Images::App,
        crate::platform::PlatformType::Recents => crate::assets::Images::Recent,
    };

    let y = y + MARGIN * 2.;
    let i = crate::assets::request_preloaded_image(piet, image);
    i.render(piet, Rect::from((Point::new(MARGIN, y), Size::new(icon_size, icon_size))));
    
    let y = y + icon_size;
    piet.stroke(Line::new(Point::new(icon_size + MARGIN * 2., y), Point::new(width - MARGIN, y)), &get_font_color(&state.settings), 2.);
    
    y + MARGIN
}