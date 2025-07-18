use crate::assets::{AssetKey, AssetSlot};
use crate::job_system::ThreadSafeJobQueue;
use crate::pooled_cache::PooledCache;
use crate::settings::{SettingNames, SettingsFile};
use crate::ui::{change_brightness, get_drawable_text};
use crate::utils::PhysicalSize;
use crate::{LogicalPosition, Rect};
use speedy2d::color::Color;
use std::cell::RefCell;
use std::collections::HashMap;
use std::ops::{Deref, DerefMut};
use yaffe_lib::SettingValue;

pub struct Graphics {
    pub graphics_ptr: *mut speedy2d::Graphics2D,
    pub queue: ThreadSafeJobQueue,
    pub scale_factor: f32,
    pub bounds: Rect,
    cached_settings: HashMap<SettingNames, SettingValue>,
    pub asset_cache: RefCell<PooledCache<32, AssetKey, AssetSlot>>,
}
impl Graphics {
    pub fn new(queue: ThreadSafeJobQueue) -> Graphics {
        Graphics {
            graphics_ptr: std::ptr::null_mut(),
            queue,
            scale_factor: 0.,
            bounds: Rect::from_tuples((0., 0.), (0., 0.)),
            cached_settings: HashMap::new(),
            asset_cache: RefCell::new(PooledCache::new()),
        }
    }
    pub unsafe fn set_frame(&mut self, graphics: &mut speedy2d::Graphics2D, scale_factor: f32, size: PhysicalSize) {
        self.graphics_ptr = graphics;
        self.scale_factor = scale_factor;
        self.bounds = Rect::new(LogicalPosition::new(0., 0.), size.to_logical(scale_factor));
    }
    pub fn cache_settings(&mut self, settings: &SettingsFile) {
        for s in [SettingNames::InfoFontSize, SettingNames::LightShadeFactor, SettingNames::DarkShadeFactor] {
            self.cached_settings.insert(s, SettingValue::F32(settings.get_f32(s)));
        }
        for s in [SettingNames::FontColor, SettingNames::AccentColor] {
            self.cached_settings.insert(s, SettingValue::Tuple(settings.get_tuple(s)));
        }
    }

    pub fn dark_shade_factor(&self) -> f32 {
        let SettingValue::F32(c) = self.cached_settings[&SettingNames::DarkShadeFactor] else {
            unreachable!();
        };
        c
    }
    pub fn light_shade_factor(&self) -> f32 {
        let SettingValue::F32(c) = self.cached_settings[&SettingNames::LightShadeFactor] else {
            unreachable!();
        };
        c
    }
    pub fn accent_color(&self) -> Color {
        let SettingValue::Tuple(c) = self.cached_settings[&SettingNames::AccentColor] else {
            unreachable!();
        };
        Color::from_rgba(c.0, c.1, c.2, c.3)
    }
    pub fn font_color(&self) -> Color {
        let SettingValue::Tuple(c) = self.cached_settings[&SettingNames::FontColor] else {
            unreachable!();
        };
        Color::from_rgba(c.0, c.1, c.2, c.3)
    }
    pub fn font_size(&self) -> f32 {
        let SettingValue::F32(s) = self.cached_settings[&SettingNames::InfoFontSize] else {
            unreachable!();
        };
        s * self.scale_factor
    }
    pub fn title_font_size(&self) -> f32 { self.font_size() * 1.25 }
    pub fn font_unfocused_color(&self) -> Color { change_brightness(&self.font_color(), -0.5) }
    pub fn accent_unfocused_color(&self) -> Color { change_brightness(&self.accent_color(), -0.3) }

    pub fn draw_rectangle(&mut self, rect: Rect, color: Color) {
        let graphics = unsafe { &mut *self.graphics_ptr };
        graphics.draw_rectangle(rect.to_physical(self.scale_factor), color);
    }
    pub fn draw_text(&mut self, position: LogicalPosition, color: Color, text: &speedy2d::font::FormattedTextBlock) {
        let graphics = unsafe { &mut *self.graphics_ptr };
        graphics.draw_text(position.to_physical(self.scale_factor), color, text);
    }
    pub fn simple_text(&mut self, position: LogicalPosition, text: &str) {
        let graphics = unsafe { &mut *self.graphics_ptr };
        let text = &get_drawable_text(self, self.font_size(), text);
        graphics.draw_text(position.to_physical(self.scale_factor), self.font_color(), text);
    }
    pub fn draw_line(&mut self, pos1: LogicalPosition, pos2: LogicalPosition, width: f32, color: Color) {
        let graphics = unsafe { &mut *self.graphics_ptr };
        graphics.draw_line(pos1.to_physical(self.scale_factor), pos2.to_physical(self.scale_factor), width, color);
    }
    pub fn draw_text_cropped(
        &mut self,
        position: LogicalPosition,
        rect: Rect,
        color: Color,
        text: &speedy2d::font::FormattedTextBlock,
    ) {
        let graphics = unsafe { &mut *self.graphics_ptr };
        graphics.draw_text_cropped(
            position.to_physical(self.scale_factor),
            rect.to_physical(self.scale_factor),
            color,
            text,
        );
    }
    pub fn draw_image(&mut self, rect: crate::Rect, image: crate::assets::Images) {
        let graphics = unsafe { &mut *self.graphics_ptr };
        let rect = rect.to_physical(self.scale_factor);
        if let Some(i) = self.request_image(image) {
            if let Some(b) = &i.bounds {
                graphics.draw_rectangle_image_subset_tinted(rect, Color::WHITE, b.clone(), &i.image);
            } else {
                graphics.draw_rectangle_image(rect, &i.image);
            }
        }
    }

    pub fn draw_image_tinted(&mut self, color: Color, rect: crate::Rect, image: crate::assets::Images) {
        let graphics = unsafe { &mut *self.graphics_ptr };
        let rect = rect.to_physical(self.scale_factor);
        if let Some(i) = self.request_image(image) {
            graphics.draw_rectangle_image_tinted(rect, color, &i.image);
        }
    }

    pub fn draw_asset_image(&mut self, rect: crate::Rect, image: &crate::assets::AssetKey) {
        let graphics = unsafe { &mut *self.graphics_ptr };
        let rect = rect.to_physical(self.scale_factor);
        if let Some(i) = self.request_asset_image(image) {
            if let Some(b) = &i.bounds {
                graphics.draw_rectangle_image_subset_tinted(rect, Color::WHITE, b.clone(), &i.image);
            } else {
                graphics.draw_rectangle_image(rect, &i.image);
            }
        }
    }
}
impl Deref for Graphics {
    type Target = speedy2d::Graphics2D;
    fn deref(&self) -> &Self::Target { unsafe { &*self.graphics_ptr } }
}
impl DerefMut for Graphics {
    fn deref_mut(&mut self) -> &mut Self::Target { unsafe { &mut *self.graphics_ptr } }
}
