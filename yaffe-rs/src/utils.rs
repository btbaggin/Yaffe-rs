use std::process::Stdio;

use speedy2d::dimen::Vector2;

pub type LogicalSize = LogicalPosition;
pub type PhysicalSize = PhysicalPosition;

#[derive(Clone, Copy, Debug)]
pub struct LogicalPosition {
    pub x: f32,
    pub y: f32,
}

impl LogicalPosition {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self { LogicalSize { x, y } }
}
impl LogicalPosition {
    #[inline]
    pub fn to_physical(self, scale_factor: f32) -> PhysicalPosition {
        let x = self.x * scale_factor;
        let y = self.y * scale_factor;
        PhysicalPosition::new(x, y)
    }
}
impl std::ops::Add for LogicalPosition {
    type Output = LogicalPosition;

    fn add(self, other: LogicalPosition) -> Self { LogicalPosition::new(self.x + other.x, self.y + other.y) }
}
impl std::ops::Sub for LogicalPosition {
    type Output = LogicalPosition;

    fn sub(self, other: LogicalPosition) -> Self { LogicalPosition::new(self.x - other.x, self.y - other.y) }
}
impl std::ops::Div<f32> for LogicalPosition {
    type Output = LogicalPosition;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output { LogicalPosition::new(self.x / rhs, self.y / rhs) }
}
impl std::ops::Mul<f32> for LogicalPosition {
    type Output = LogicalPosition;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output { LogicalPosition::new(self.x * rhs, self.y * rhs) }
}

#[derive(Clone, Copy)]
pub struct PhysicalPosition {
    pub x: f32,
    pub y: f32,
}

impl PhysicalPosition {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self { PhysicalPosition { x, y } }
}
impl PhysicalPosition {
    #[inline]
    pub fn from_logical<T: Into<LogicalPosition>>(logical: T, scale_factor: f32) -> Self {
        logical.into().to_physical(scale_factor)
    }

    #[inline]
    pub fn to_logical(self, scale_factor: f32) -> LogicalPosition {
        let x = self.x / scale_factor;
        let y = self.y / scale_factor;
        LogicalPosition::new(x, y)
    }
}
impl From<PhysicalPosition> for Vector2<f32> {
    fn from(other: PhysicalPosition) -> Self { Vector2::new(other.x, other.y) }
}

#[derive(Copy, Clone, Debug)]
pub struct Rect {
    top_left: LogicalPosition,
    bottom_right: LogicalPosition,
}

pub type PhysicalRect = speedy2d::shape::Rectangle<f32>;

impl Rect {
    pub fn left(&self) -> f32 { self.top_left.x }
    pub fn right(&self) -> f32 { self.bottom_right.x }
    pub fn top(&self) -> f32 { self.top_left.y }
    pub fn bottom(&self) -> f32 { self.bottom_right.y }
    pub fn top_left(&self) -> &LogicalPosition { &self.top_left }
    pub fn bottom_right(&self) -> &LogicalPosition { &self.bottom_right }
    pub fn width(&self) -> f32 { self.bottom_right.x - self.top_left.x }
    pub fn height(&self) -> f32 { self.bottom_right.y - self.top_left.y }
    pub fn size(&self) -> LogicalSize { LogicalSize::new(self.width(), self.height()) }

    pub fn new(top_left: LogicalPosition, bottom_right: LogicalPosition) -> Rect { Rect { top_left, bottom_right } }
    pub fn from_tuples(top_left: (f32, f32), bottom_right: (f32, f32)) -> Rect {
        Rect {
            top_left: LogicalPosition::new(top_left.0, top_left.1),
            bottom_right: LogicalPosition::new(bottom_right.0, bottom_right.1),
        }
    }
    pub fn point_and_size(pos: LogicalPosition, size: LogicalSize) -> Self { Rect::new(pos, pos + size) }

    pub fn percent(parent: Rect, percent: LogicalSize) -> Self {
        let pos = *parent.top_left();
        let size = parent.size();
        let size = LogicalSize::new(size.x * percent.x, size.y * percent.y);
        Rect::new(pos, pos + size)
    }

    pub fn to_physical(self, scale_factor: f32) -> PhysicalRect {
        let top_left = self.top_left.to_physical(scale_factor);
        let bottom_right = self.bottom_right.to_physical(scale_factor);

        PhysicalRect::new(top_left.into(), bottom_right.into())
    }
}

pub trait Transparent {
    fn with_alpha(&self, alpha: f32) -> Self;
}
impl Transparent for speedy2d::color::Color {
    fn with_alpha(&self, alpha: f32) -> Self { speedy2d::color::Color::from_rgba(self.r(), self.g(), self.b(), alpha) }
}

pub trait ScaleFactor {
    fn to_logical(&self, graphics: &crate::Graphics) -> f32;
    fn to_physical(&self, graphics: &crate::Graphics) -> f32;
}
impl ScaleFactor for f32 {
    fn to_logical(&self, graphics: &crate::Graphics) -> f32 { self / graphics.scale_factor }
    fn to_physical(&self, graphics: &crate::Graphics) -> f32 { self * graphics.scale_factor }
}

pub fn yaffe_helper(action: &str, args: &[&str]) -> std::io::Result<std::process::Child> {
    let helper_path = append_app_ext("./yaffe-helper");
    std::process::Command::new(helper_path).arg(action).args(args).stdout(Stdio::piped()).spawn()
}

pub fn append_app_ext(path: &str) -> String {
    let app_ext = crate::os::app_ext();
    if !app_ext.is_empty() {
        return format!("{path}.{app_ext}");
    }
    path.to_string()
}
