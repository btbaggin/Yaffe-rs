use crate::{Actions, LogicalSize, LogicalPosition, Rect, Graphics};
use crate::ui::{AnimationManager, WidgetId, UiElement, LayoutElement, Color};

struct ContainerChild<T, D> {
    element: Box<dyn UiElement<T, D>>,
    size: ContainerSize,
    realized_size: f32,
}

#[derive(Clone, Copy)]
pub enum ContainerSize {
    Percent(f32),
    Fixed(f32),
    Fill,
    Shrink
}

#[derive(Debug)]
enum FlexDirection {
    Row,
    Column,
}

enum BackgroundType {
    Image(crate::assets::Images),
    Color(Color),
    None,
}

pub struct UiContainer<T: 'static, D: 'static> {
    position: LogicalPosition,
    size: LogicalSize,
    id: WidgetId,
    children: Vec<ContainerChild<T, D>>,
    background: BackgroundType,
    direction: FlexDirection,
    margin: f32,
}
impl<T, D> LayoutElement for UiContainer<T, D> {
    fn layout(&self) -> Rect { Rect::new(self.position, self.position + self.size) }
    fn set_layout(&mut self, layout: Rect) {
        self.position = *layout.top_left();
        self.size = layout.size();
    }
    fn get_id(&self) -> WidgetId { self.id }
    fn as_any(&self) -> &dyn std::any::Any { self }
    fn as_any_mut(&mut self) -> &mut dyn std::any::Any { self }
}
impl<T, D> UiContainer<T, D> {
    pub fn row() -> UiContainer<T, D> {
        UiContainer {
            position: LogicalPosition::new(0., 0.),
            size: LogicalSize::new(0., 0.),
            id: WidgetId::random(),
            children: vec![],
            background: BackgroundType::None,
            direction: FlexDirection::Row,
            margin: 5.
        }
    }

    pub fn column() -> UiContainer<T, D> {
        UiContainer {
            position: LogicalPosition::new(0., 0.),
            size: LogicalSize::new(0., 0.),
            id: WidgetId::random(),
            children: vec![],
            background: BackgroundType::None,
            direction: FlexDirection::Column,
            margin: 5.
        }
    }

    pub fn background_image(&mut self, image: crate::assets::Images) -> &mut Self {
        self.background = BackgroundType::Image(image);
        self
    }

    pub fn background_color(&mut self, color: Color) -> &mut Self {
        self.background = BackgroundType::Color(color);
        self
    }

    pub fn margin(&mut self, margin: f32) -> &mut Self {
        self.margin = margin;
        self
    }

    pub fn get_child(&self, index: usize) -> &Box<dyn UiElement<T, D>> {
        &self.children[index].element
    }

    pub fn add_child(&mut self, child: impl UiElement<T, D> + 'static, size: ContainerSize) -> &mut Self {
        let child = ContainerChild { element: Box::new(child), size, realized_size: 0. };
        self.children.push(child);
        self
    }

    pub fn with_child(&mut self, child: UiContainer<T, D>, size: ContainerSize) -> &mut UiContainer<T, D> {
        self.add_child(child, size);

        let count = self.children.len();
        self.children[count - 1].element.as_mut().as_any_mut().downcast_mut::<UiContainer<T, D>>().unwrap()
    }

    pub fn find_widget(&self, widget_id: WidgetId) -> Option<&dyn UiElement<T, D>> {
        // Check if the current container matches the widget_id
        if self.get_id() == widget_id {
            return Some(self);
        }

        // Recursively search in children
        for child in &self.children {
            if child.element.get_id() == widget_id {
                return Some(child.element.as_ref());
            } else if let Some(container) = child.element.as_any().downcast_ref::<UiContainer<T, D>>() {
                if let Some(found) = container.find_widget(widget_id) {
                    return Some(found);
                }
            }
        }

        None
    }

    pub fn find_widget_mut(&mut self, widget_id: WidgetId) -> Option<&mut dyn UiElement<T, D>> {
        // Check if the current container matches the widget_id
        if self.get_id() == widget_id {
            return Some(self);
        }

        // Recursively search in children
        for child in &mut self.children {
            if child.element.get_id() == widget_id {
                return Some(child.element.as_mut());
            } else if let Some(container) = child.element.as_any_mut().downcast_mut::<UiContainer<T, D>>() {
                if let Some(found) = container.find_widget_mut(widget_id) {
                    return Some(found);
                }
            }
        }

        None
    }

    pub fn replace_child(&mut self, widget_id: WidgetId, child: impl UiElement<T, D> + 'static) {
        let child = Box::new(child);
        self.replace_child_boxed(widget_id, child);
    }

    fn replace_child_boxed(&mut self, widget_id: WidgetId, child: Box<dyn UiElement<T, D>>) {
        // Check direct children
        for (i, c) in self.children.iter_mut().enumerate() {
            if c.element.get_id() == widget_id {
                let size = c.size;
                self.children[i] = ContainerChild { 
                    element: child, 
                    size, 
                    realized_size: 0. 
                };
                return;
            }
        }
        
        // Search recursively
        for c in &mut self.children {
            if let Some(container) = c.element.as_any_mut().downcast_mut::<UiContainer<T, D>>() {
                container.replace_child_boxed(widget_id, child);
                return;
            }
        }
    }

    fn calc_container_size(&mut self, graphics: &mut Graphics) -> LogicalSize {
        let parent_size = graphics.bounds.size();
        let mut total_fixed = 0.0;
        let mut total_percent = 0.0;
        let mut total_shrink = 0.0;
        let mut fill_count = 0;

        let margin_size = self.children.len() as f32 * 5.;

        let total = match self.direction {
            FlexDirection::Row => parent_size.x,
            FlexDirection::Column => parent_size.y,
        } - margin_size;

        // Calculate the total fixed, percent, and shrink sizes, and count the fill elements
        for child in &mut self.children {
            match child.size {
                ContainerSize::Fixed(size) => {
                    total_fixed += size;
                    child.realized_size = size;
                }
                ContainerSize::Percent(percent) => {
                    total_percent += total * percent;
                    child.realized_size = total * percent;
                }
                ContainerSize::Fill => {
                    fill_count += 1;
                }
                ContainerSize::Shrink => {
                    let size = child.element.calc_size(graphics);
                    let size = match self.direction {
                        FlexDirection::Row => size.x,
                        FlexDirection::Column => size.y,
                    };
                    total_shrink += size;
                    child.realized_size = size;
                }
            }
        }

        // Calculate the remaining space for Fill elements
        let available_space = total - total_fixed - total_percent - total_shrink;
        let fill_size = if fill_count > 0 {
            available_space / fill_count as f32
        } else {
            0.0
        };

        for child in &mut self.children {
            if let ContainerSize::Fill = child.size {
                child.realized_size = fill_size;
            }
        }

        // Calculate the total size of the container
        let total_size = match self.direction {
            FlexDirection::Row => LogicalSize::new(
                total_fixed + total_percent + total_shrink + (fill_size * fill_count as f32) + margin_size,
                parent_size.y,
            ),
            FlexDirection::Column => LogicalSize::new(
                parent_size.x,
                total_fixed + total_percent + total_shrink + (fill_size * fill_count as f32) + margin_size,
            ),
        };

        total_size
    }

    pub fn move_focus(&mut self, current_focus: Option<WidgetId>, next: bool) -> Option<WidgetId> {
        //Try to find current focus
        //Move index based on index and if it exists
        let child_count = self.children.len() as isize;
        let index = match current_focus {
            None => {
                if next { 0isize } else { child_count - 1 }
            }
            Some(_) => {
                let index = self.children.iter().position(|c| Some(c.element.get_id()) == current_focus);
                if let Some(index) = index {
                    let index = index as isize;
                    if next { index + 1 } else { index - 1 }
                } else {
                    child_count
                }
            }
        };

        if index >= 0 && index < child_count {
            Some(self.children[index as usize].element.get_id())
        } else {
            None
        }
    }
}

impl<T: 'static, D: 'static> UiElement<T, D> for UiContainer<T, D> {
    fn calc_size(&mut self, graphics: &mut Graphics) -> LogicalSize { self.calc_container_size(graphics) }

    fn render(&mut self, graphics: &mut Graphics, state: &T, current_focus: &WidgetId) {
        let total_size = self.calc_container_size(graphics);

        // TODO this is shit
        // TODO justification
        let layout = Rect::point_and_size(*graphics.bounds.top_left(), total_size);
        graphics.bounds = layout;
        self.set_layout(layout);

        match self.background {
            BackgroundType::Image(i) => {
                // TODO tinted is jank
                let base = graphics.accent_color();
                graphics.draw_image_tinted(base, graphics.bounds, i);
            },
            BackgroundType::Color(c) => graphics.draw_rectangle(graphics.bounds, c),
            BackgroundType::None => {}
        }

        for child in &mut self.children {
            let (width, height, x_offset, y_offset) = match self.direction {
                FlexDirection::Row => {
                    let width = child.realized_size;
                    let height = graphics.bounds.height();
                    (width, height, width + self.margin, 0.)
                }
                FlexDirection::Column => {
                    let width = graphics.bounds.width();
                    let height = child.realized_size;
                    (width, height, 0., height + self.margin)
                }
            };

            let origin = *graphics.bounds.top_left();
            let size = graphics.bounds.size();
            child.element.set_layout(Rect::point_and_size(origin, LogicalSize::new(width, height)));

            child.element.render(graphics, state, current_focus);
            graphics.bounds = Rect::point_and_size(
                origin + LogicalPosition::new(x_offset, y_offset),
                LogicalSize::new(size.x - x_offset, size.y - y_offset),
            );
        }
    }

    fn action(&mut self, state: &mut T, animations: &mut AnimationManager, action: &Actions, handler: &mut D) -> bool {
        for child in &mut self.children {
            if child.element.action(state, animations, action, handler) {
                return true;
            }
        }
        false
    }
}