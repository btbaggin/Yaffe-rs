use crate::assets::Images;
use crate::ui::MODAL_BACKGROUND;
use crate::{LogicalPosition, LogicalSize, PhysicalSize, Rect, ScaleFactor, Transparent, YaffeState};

pub const ANIMATION_TIME: f32 = 0.25;
const SELECTED_SCALAR: f32 = 0.2;
const ROM_OUTLINE_SIZE: f32 = 7.5;

const VISIBLE_FLAG: u8 = 0x01;

pub struct AppTile {
    index: usize,
    flags: u8,
    pub position: LogicalPosition,
    pub size: LogicalSize,
}
impl AppTile {
    pub fn new(index: usize) -> AppTile {
        AppTile { index, flags: VISIBLE_FLAG, position: LogicalPosition::new(0., 0.), size: LogicalSize::new(0., 0.) }
    }

    pub fn intersects(&self, rect: &crate::Rect) -> bool {
        let x1_y1 = self.position;
        let x2_y2 = self.position + self.size;
        let b_x1_y1 = rect.top_left();
        let b_x2_y2 = rect.bottom_right();

        x1_y1.x < b_x2_y2.x && x2_y2.x > b_x1_y1.x && x1_y1.y < b_x2_y2.y && x2_y2.y > b_x1_y1.y
    }

    pub fn is_visible(&self) -> bool { (self.flags & VISIBLE_FLAG) != 0 }

    pub fn apply_filter(&mut self, filter: &Option<crate::state::MetadataSearch>, apps: &[crate::Tile]) {
        fn set_visible(flags: u8, visible: bool) -> u8 {
            if visible {
                flags | VISIBLE_FLAG
            } else {
                flags & !VISIBLE_FLAG
            }
        }
        let exe = &apps[self.index];
        if let Some(filter) = filter {
            self.flags = set_visible(self.flags, filter.item_is_visible(exe))
        } else {
            self.flags = VISIBLE_FLAG;
        }
    }

    pub fn render(&mut self, focused: bool, animation: f32, exe: &crate::Tile, graphics: &mut crate::Graphics) {
        if !self.is_visible() {
            return;
        }

        let mut target_size = self.size;
        let mut position = self.position;

        if focused {
            //Have alpha fade in as the time grows to full size
            let alpha = f32::powf(animation, 2.);
            let font_size = graphics.font_size();

            target_size = target_size * (1. + animation * SELECTED_SCALAR);
            position = position - (target_size - self.size) / 2.;

            //Position of the text and buttons for the focused game
            let mut menu_position = LogicalPosition::new(position.x + target_size.x, position.y + target_size.y + 2.);

            let name = crate::ui::get_drawable_text_with_wrap(graphics, font_size, &exe.name, target_size.x);
            let mut height = 0.;
            let info = crate::ui::get_drawable_text(graphics, font_size, "Info");
            let run = crate::ui::get_drawable_text(graphics, font_size, "Run");

            let physical_width = target_size.x.to_physical(graphics);
            let options_width = info.width() + run.width() + font_size * 2. + 5. * 2.;

            let lines: Vec<&std::rc::Rc<speedy2d::font::FormattedTextLine>> = name.iter_lines().collect();
            let line_count = lines.len();
            for (line_number, line) in lines.into_iter().enumerate() {
                let line_height = line.height().to_logical(graphics);
                //We need to move the menu down while it isnt the last line
                //or the line is big enough where the menu won't fit
                if line_number < line_count - 1 {
                    menu_position.y += line_height;
                } else if line.width() > physical_width - options_width {
                    menu_position.y += line_height;
                    height += line_height;
                }
                height += line_height;
            }

            //Outline background
            let rect_start = position - LogicalSize::new(ROM_OUTLINE_SIZE, ROM_OUTLINE_SIZE);
            let rect_size =
                LogicalSize::new(target_size.x + ROM_OUTLINE_SIZE * 2., target_size.y + height + ROM_OUTLINE_SIZE * 2.);
            graphics
                .draw_rectangle(Rect::point_and_size(rect_start, rect_size), MODAL_BACKGROUND.with_alpha(alpha * 0.94));

            graphics.draw_text(
                LogicalPosition::new(position.x, position.y + target_size.y),
                graphics.font_color().with_alpha(alpha),
                &name,
            );

            //Help
            menu_position = crate::ui::right_aligned_text(
                graphics,
                menu_position,
                Some(Images::ButtonX),
                graphics.font_color().with_alpha(alpha),
                info,
            );
            menu_position.x -= 5.;

            crate::ui::right_aligned_text(
                graphics,
                menu_position,
                Some(Images::ButtonA),
                graphics.font_color().with_alpha(alpha),
                run,
            );
        }

        if graphics.request_asset_image(&exe.boxart.clone()).is_some() {
            graphics.draw_asset_image(Rect::point_and_size(position, target_size), &exe.boxart.clone());
        } else {
            graphics.draw_image(Rect::point_and_size(position, target_size), Images::Placeholder);
        }
    }

    pub fn get_image(&self, state: &YaffeState) -> crate::assets::AssetKey {
        let p = state.get_selected_group();
        let exe = &p.tiles[self.index];
        exe.boxart.clone()
    }

    pub fn get_image_size(&self, state: &YaffeState, graphics: &mut crate::Graphics) -> PhysicalSize {
        let slot = self.get_image(state);

        if let Some(i) = graphics.request_asset_image(&slot) {
            return i.size();
        }

        graphics.request_image(Images::Placeholder).unwrap().size()
    }
}
