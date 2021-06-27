use druid_shell::kurbo::{Rect, Point, Line};
use druid_shell::piet::{Piet, RenderContext};
use crate::{font, colors::get_font_color, widgets::get_drawable_text, YaffeState, Actions};
use crate::modals::*;
use crate::logger::UserMessage;

const STARTUP_TASK: &str = "Yaffe";

pub struct SettingsModal {
    run_at_startup: bool,
}
impl SettingsModal {
    pub fn new() -> SettingsModal {
        let set = match crate::platform_code::get_run_at_startup(STARTUP_TASK) {
            Ok(v) => v,
            Err(e) => {
                crate::logger::log_entry_with_message(crate::logger::LogTypes::Error, e, "Unable to get if Yaffe runs at startup");
                false
            }
        };
        SettingsModal { run_at_startup: set, }
    }
}

impl ModalContent for SettingsModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_height(&self) -> f64 {
        crate::font::FONT_SIZE + crate::ui::MARGIN
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rect, piet: &mut Piet) {
        let pos = rect.origin();
        let label = get_drawable_text(piet, font::FONT_SIZE, "Run At Startup", get_font_color(settings));
        piet.draw_text(&label, pos); 

        let min = Point::new(pos.x + crate::ui::LABEL_SIZE, pos.y);
        let max = Point::new(pos.x + crate::ui::LABEL_SIZE + 24., pos.y + 24.);
        let checkbox = Rect::from((min, max));
        
        let base = crate::colors::get_accent_color(settings);
        let factor = settings.get_f64(crate::SettingNames::DarkShadeFactor);
        piet.fill(checkbox, &crate::colors::change_brightness(&base, factor));
        if self.run_at_startup {
            piet.stroke(Line::new(min, max), &base, 2.);
            piet.stroke(Line::new(Point::new(min.x, max.y), Point::new(max.x, min.y)), &base, 2.);
        }
    }

    fn action(&mut self, action: &Actions, _: &mut DeferredModalAction) -> ModalResult {
        match action {
            Actions::Select => {
                self.run_at_startup = !self.run_at_startup;
                ModalResult::None
            }
            _ => default_modal_action(action),
        }
    }
}

pub fn on_settings_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<SettingsModal>().unwrap();
        crate::platform_code::set_run_at_startup(STARTUP_TASK, content.run_at_startup).display_failure("Unable to set Yaffe to run at startup", state);
    }
}