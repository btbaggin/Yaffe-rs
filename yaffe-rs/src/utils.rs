use speedy2d::dimen::Vector2;

pub type LogicalSize  = LogicalPosition;
pub type PhysicalSize = PhysicalPosition;

#[derive(Clone, Copy)]
pub struct LogicalPosition {
    pub x: f32,
    pub y: f32,
}

impl LogicalPosition {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        LogicalSize { x, y }
    }
}
impl LogicalPosition {
    #[inline]
    pub fn from_physical<T: Into<PhysicalPosition>>(
        physical: T,
        scale_factor: f32,
    ) -> Self {
        physical.into().to_logical(scale_factor)
    }
    
    #[inline]
    pub fn to_physical(&self, scale_factor: f32) -> PhysicalPosition {
        let x = self.x * scale_factor;
        let y = self.y * scale_factor;
        PhysicalPosition::new(x, y)
    }
}
impl From<LogicalPosition> for Vector2<f32> {
    fn from(other: LogicalPosition) -> Self {
        Vector2::new(other.x, other.y)
    }
}
impl std::ops::Add for LogicalPosition {
    type Output = LogicalPosition;

    fn add(self, other: LogicalPosition) -> Self {
        LogicalPosition::new(self.x + other.x, self.y + other.y)
    }
}
impl std::ops::Sub for LogicalPosition {
    type Output = LogicalPosition;

    fn sub(self, other: LogicalPosition) -> Self {
        LogicalPosition::new(self.x - other.x, self.y - other.y)
    }
}
impl std::ops::Div<f32> for LogicalPosition {
    type Output = LogicalPosition;
    #[inline]
    fn div(self, rhs: f32) -> Self::Output {
        LogicalPosition::new(self.x / rhs, self.y / rhs)
    }
}
impl std::ops::Mul<f32> for LogicalPosition {
    type Output = LogicalPosition;
    #[inline]
    fn mul(self, rhs: f32) -> Self::Output {
        LogicalPosition::new(self.x * rhs, self.y * rhs)
    }
}

#[derive(Clone, Copy)]
pub struct PhysicalPosition {
    pub x: f32,
    pub y: f32,
}

impl PhysicalPosition {
    #[inline]
    pub const fn new(x: f32, y: f32) -> Self {
        PhysicalPosition { x, y }
    }
}
impl PhysicalPosition {
    #[inline]
    pub fn from_logical<T: Into<LogicalPosition>>(logical: T, scale_factor: f32) -> Self {
        logical.into().to_physical(scale_factor)
    }
    
    #[inline]
    pub fn to_logical(&self, scale_factor: f32) -> LogicalPosition {
        let x = self.x / scale_factor;
        let y = self.y / scale_factor;
        LogicalPosition::new(x, y)
    }
}

// #[derive(Clone, Copy)]
// pub struct Rect {
//     top_left: LogicalPosition,
//     bottom_right: LogicalPosition,
// }

// impl Rect {
//     pub fn left(&self) -> f32 { self.top_left.x }
//     pub fn right(&self) -> f32 { self.bottom_right.x }
//     pub fn top(&self) -> f32 { self.top_left.y }
//     pub fn bottom(&self) -> f32 { self.bottom_right.y }
//     pub fn width(&self) -> f32 { self.bottom_right.x - self.top_left.x }
//     pub fn height(&self) -> f32 { self.bottom_right.y - self.top_left.y }
//     pub fn top_left(&self) -> &LogicalPosition {
//         &self.top_left
//     }
//     pub fn bottom_right(&self) -> &LogicalPosition {
//         &self.bottom_right
//     }
//     pub fn point_and_size(pos: LogicalPosition, size: LogicalSize) -> Self { 
//         Rect::new(pos, pos + size) 
//     }
//     pub fn new(top_left: LogicalPosition, bottom_right: LogicalPosition) -> Self {
//         Rect { top_left, bottom_right }
//     }
// }
// impl From<Rect> for speedy2d::shape::Rectangle<f32> {
//     fn from(value: Rect) -> speedy2d::shape::Rectangle<f32> {
//         speedy2d::shape::Rectangle::from_tuples((value.top_left.x, value.top_left.y), (value.bottom_right.x, value.bottom_right.y))
//     }
// }

#[derive(Copy, Clone)]
pub struct Rect {
    top_left: LogicalPosition,
    bottom_right: LogicalPosition,
}

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

    pub fn new(top_left: LogicalPosition, bottom_right: LogicalPosition) -> Rect {
        Rect { top_left, bottom_right }
    }
    pub fn from_tuples(top_left: (f32, f32), bottom_right: (f32, f32)) -> Rect {
        Rect { 
            top_left: LogicalPosition::new(top_left.0, top_left.1), 
            bottom_right: LogicalPosition::new(bottom_right.0, bottom_right.1),
        }
    }
    pub fn point_and_size(pos: LogicalPosition, size: LogicalSize) -> Self { Rect::new(pos, pos + size) }
}
impl From<Rect> for speedy2d::shape::Rectangle<f32> {
    fn from(other: Rect) -> Self {
        speedy2d::shape::Rectangle::from_tuples((other.top_left.x, other.top_left.y), (other.bottom_right.x, other.bottom_right.y))
    }
}
impl From<&Rect> for speedy2d::shape::Rectangle<f32> {
    fn from(other: &Rect) -> Self {
        speedy2d::shape::Rectangle::from_tuples((other.top_left.x, other.top_left.y), (other.bottom_right.x, other.bottom_right.y))
    }
}
// impl Rect for speedy2d::shape::Rectangle {
//     fn left(&self) -> f32 { self.top_left().x }
//     fn right(&self) -> f32 { self.bottom_right().x }
//     fn top(&self) -> f32 { self.top_left().y }
//     fn bottom(&self) -> f32 { self.bottom_right().y }
//     fn point_and_size(pos: LogicalPosition, size: LogicalSize) -> Self { speedy2d::shape::Rectangle::new(pos.into(), (pos + size).into()) }
// }

pub trait Transparent {
    fn with_alpha(&self, alpha: f32) -> Self;
}
impl Transparent for speedy2d::color::Color {
    fn with_alpha(&self, alpha: f32) -> Self {
        speedy2d::color::Color::from_rgba(self.r(), self.g(), self.b(), alpha)
    }
}