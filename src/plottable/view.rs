use raylib::math::Vector2;

use crate::plottable::point::Point;

#[derive(Default, Clone, Copy, Debug)]
pub struct Offsets {
    pub(crate) offset_x: f32,
    pub(crate) offset_y: f32,
}

impl Offsets {
    pub fn new(offset_x: f32, offset_y: f32) -> Self {
        Self { offset_x, offset_y }
    }
    pub fn offset_point(&self, point: &Point) -> Point {
        Point {
            x: point.x + self.offset_x,
            y: point.y + self.offset_y,
        }
    }
}

impl From<(f32, f32)> for Offsets {
    fn from(value: (f32, f32)) -> Self {
        Self {
            offset_x: value.0,
            offset_y: value.1,
        }
    }
}
impl From<raylib::math::Vector2> for Offsets {
    fn from(value: Vector2) -> Self {
        Self {
            offset_x: value.x,
            offset_y: value.y,
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub struct BBox {
    pub maximum: Point,
    pub minimum: Point,
}

impl BBox {
    pub fn width(&self) -> f32 {
        (self.maximum.x - self.minimum.x).abs()
    }

    pub fn heigth(&self) -> f32 {
        (self.maximum.y - self.minimum.y).abs()
    }
}

impl Default for BBox {
    fn default() -> Self {
        Self {
            minimum: Point { x: 0.0, y: 0.0 },
            maximum: Point {
                x: crate::WIDTH as f32,
                y: crate::HEIGHT as f32,
            },
        }
    }
}

impl From<(Point, Point)> for BBox {
    fn from(value: (Point, Point)) -> Self {
        Self {
            minimum: value.0,
            maximum: value.1,
        }
    }
}
pub fn map_range(value: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    (value - in_min) / (in_max - in_min) * (out_max - out_min) + out_min
}
#[derive(Debug, Clone, Copy)]
pub struct ViewTransformer {
    pub data_bounds: BBox,
    pub screen_bounds: BBox,
    pub offset: Offsets,
}

impl ViewTransformer {
    pub fn new(data_bounds: BBox, screen_bounds: BBox, offset: Offsets) -> Self {
        Self {
            data_bounds,
            screen_bounds,
            offset,
        }
    }

    /// Linearly maps a value from one range to another
    fn map_val(val: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
        if (in_max - in_min).abs() < f32::EPSILON {
            return out_min; // Avoid division by zero if range is 0
        }
        (val - in_min) / (in_max - in_min) * (out_max - out_min) + out_min
    }

    /// Converts a point from Data Space to Screen Space
    pub fn to_screen(&self, point: &Point) -> Point {
        let x = Self::map_val(
            point.x,
            self.data_bounds.minimum.x,
            self.data_bounds.maximum.x,
            self.screen_bounds.minimum.x,
            self.screen_bounds.maximum.x,
        ) + self.offset.offset_x;

        // FLIP Y-AXIS:
        // Data Min (low value) -> Screen Max (bottom of screen, high pixel count)
        // Data Max (high value) -> Screen Min (top of screen, 0)
        let y = Self::map_val(
            point.y,
            self.data_bounds.minimum.y,
            self.data_bounds.maximum.y,
            self.screen_bounds.maximum.y,
            self.screen_bounds.minimum.y,
        ) + self.offset.offset_y;

        Point { x, y }
    }
}
