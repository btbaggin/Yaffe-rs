use speedy2d::Graphics2D;
use speedy2d::shape::Rectangle;
use crate::{font, widgets::get_drawable_text, YaffeState, Actions, Rect, V2};
use crate::modals::*;
use crate::colors::*;
use crate::logger::{LogEntry, UserMessage};

pub struct PlatformDetailModal {
    application: bool,
    name: String,
    exe: String,
    args: String,
    folder: String,
    id: i64,
    selected_field: u32,
}
impl PlatformDetailModal {
    pub fn application() -> PlatformDetailModal {
        PlatformDetailModal { 
            application: true,
            name: String::from(""),
            exe: String::from(""),
            args: String::from(""),
            folder: String::from(""),
            id: 0,
            selected_field: 0,
        }
    }

    pub fn emulator() -> PlatformDetailModal {
        PlatformDetailModal { 
            application: false,
            name: String::from("Playstation"),
            exe: String::from(""),
            args: String::from(""),
            folder: String::from(""),
            id: 0,
            selected_field: 0,
        }
    }

    pub fn from_existing(plat: &crate::Platform) -> PlatformDetailModal {
        //This should never fail since we orignally got it from the database
        let (path, args, roms) = crate::database::get_platform_info(plat.id).log_if_fail();

        PlatformDetailModal { 
            application: plat.kind == crate::platform::PlatformType::App,
            name: plat.name.clone(),
            exe: path,
            args: args,
            folder: roms,
            id: plat.id,
            selected_field: 0,
        }
    }
}

impl ModalContent for PlatformDetailModal {
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn get_height(&self) -> f32 {
        (crate::font::FONT_SIZE + crate::ui::MARGIN) * 4.
    }

    fn action(&mut self, action: &Actions, handler: &mut crate::windowing::WindowHelper) -> modal::ModalResult {
        match action {
            Actions::Select => {
                match self.selected_field {
                    1 => handler.open_file(),
                    3 => if self.application { handler.open_file() } else { handler.open_directory() },
                    _ => {}
                }
                modal::ModalResult::None
            }
            Actions::Down => {
                if self.selected_field < 3 { self.selected_field += 1; }
                modal::ModalResult::None
            }
            Actions::Up => {
                if self.selected_field > 0 { self.selected_field -= 1; }
                modal::ModalResult::None

            }
            _ => default_modal_action(action)
        }
    }

    fn render(&self, settings: &crate::settings::SettingsFile, rect: Rectangle, piet: &mut Graphics2D) {
        let draw_rect = Rect::point_and_size(*rect.top_left(), V2::new(rect.right(), rect.top() + crate::font::FONT_SIZE));
        let draw_rect = draw_field_with_label(piet, self, settings, "Name", &self.name, 0, draw_rect);
        let draw_rect = draw_field_with_label(piet, self, settings, "Executable", &self.exe, 1, draw_rect);
        let draw_rect = draw_field_with_label(piet, self, settings, "Args", &self.args, 2, draw_rect);
        draw_field_with_label(piet, self, settings, if self.application { "Image" } else { "Folder" }, &self.folder, 3, draw_rect);
    }
}

fn draw_field_with_label(piet: &mut Graphics2D, 
                         modal: &PlatformDetailModal, 
                         settings: &crate::settings::SettingsFile, 
                         caption: &str, 
                         field: &str, 
                         index: u32, 
                         rect: Rectangle) -> Rectangle {
    //Caption
    let position = rect.top_left();
    let label = get_drawable_text(font::FONT_SIZE, caption);
    piet.draw_text(*position, get_font_color(settings), &label); 
    
    //Background rect
    let value_rect = Rectangle::from_tuples((rect.left() + crate::ui::LABEL_SIZE, rect.top()), (rect.right(), rect.bottom()));
    piet.draw_rectangle(value_rect.clone(), get_accent_unfocused_color(settings));
    if modal.selected_field == index {
        modal::outline_rectangle(piet, &value_rect, 2., get_accent_color(settings));
    }
  
    //Value
    let value = get_drawable_text(font::FONT_SIZE, field);
    piet.draw_text(V2::new(position.x + crate::ui::LABEL_SIZE, position.y), get_font_color(settings), &value);
    
    //Return drawing rect for next field
    Rectangle::from_tuples((rect.left(), rect.bottom() + crate::ui::MARGIN), (rect.right(), rect.bottom() + crate::font::FONT_SIZE + crate::ui::MARGIN))
}

pub fn on_add_platform_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<PlatformDetailModal>().unwrap();

        if content.application {
            crate::database::add_new_application(&content.name, &content.exe, &content.args, &content.folder).display_failure("Unable to add application", state);
            state.refresh_list = true;
        } else {
            let state_ptr = crate::RawDataPointer::new(state);
            let mut queue = state.queue.borrow_mut();
            queue.send(crate::JobType::SearchPlatform((state_ptr, content.name.clone(), content.exe.clone(), content.args.clone(), content.folder.clone())));  
        }
    }
}

pub fn on_update_application_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<PlatformDetailModal>().unwrap();
        state.refresh_list = true;

        if content.application {
			crate::database::update_application(&content.name, &content.exe, &content.args).display_failure("Unable to update application", state);
		} else {
			crate::database::update_platform(content.id, &content.exe, &content.args, &content.folder).display_failure("Unable to update platform", state);
		}
    }
}

pub fn on_platform_found_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<crate::modals::ListModal<crate::database::PlatformData>>().unwrap();

        let item = content.get_selected();
        crate::platform::insert_platform(state, item);
    }
}

pub fn on_game_found_close(state: &mut YaffeState, result: ModalResult, content: &Box<dyn ModalContent>) {
    if let ModalResult::Ok = result {
        let content = content.as_any().downcast_ref::<crate::modals::ListModal<crate::database::GameData>>().unwrap();

        let item = content.get_selected();
        crate::platform::insert_game(state, item);
    }
}