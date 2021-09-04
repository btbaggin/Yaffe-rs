use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::{font, colors::get_font_color, widgets::get_drawable_text, YaffeState, Actions, V2};
use crate::modals::*;
use crate::logger::UserMessage;

const STARTUP_TASK: &str = "Yaffe";

pub struct SettingsModal {
    run_at_startup: bool,
}
impl SettingsModal {
    pub fn new() -> SettingsModal {
        let set = match crate::platform_layer::get_run_at_startup(STARTUP_TASK) {
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
    fn get_height(&self) -> f32 {
        crate::font::FONT_SIZE + crate::ui::MARGIN
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rectangle, piet: &mut Graphics2D) {
        let pos = rect.top_left();
        let label = get_drawable_text(font::FONT_SIZE, "Run At Startup");
        piet.draw_text(*pos, get_font_color(settings), &label); 

        let min = V2::new(pos.x + crate::ui::LABEL_SIZE, pos.y);
        let max = V2::new(pos.x + crate::ui::LABEL_SIZE + 24., pos.y + 24.);
        let checkbox = Rectangle::new(min, min + max);
        
        let base = crate::colors::get_accent_color(settings);
        let factor = settings.get_f32(crate::SettingNames::DarkShadeFactor);
        piet.draw_rectangle(checkbox, crate::colors::change_brightness(&base, factor));
        if self.run_at_startup {
            piet.draw_line(min, max, 2., base);
            piet.draw_line(V2::new(min.x, max.y), V2::new(max.x, min.y), 2., base);
        }
    }

    fn action(&mut self, action: &Actions, _: &mut crate::windowing::WindowHelper) -> ModalResult {
        match action {
            Actions::Select => {
                self.run_at_startup = !self.run_at_startup;
                ModalResult::None
            }
            _ => default_modal_action(action),
        }
    }
}

pub fn on_settings_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>, _: &mut crate::DeferredAction) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<SettingsModal>().unwrap();
        crate::platform_layer::set_run_at_startup(STARTUP_TASK, content.run_at_startup).display_failure("Unable to set Yaffe to run at startup", state);
    }
}