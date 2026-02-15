use crate::plottable::point::Point;

// #[derive(Default, Clone, Copy, Debug)]
// pub struct Offsets {
//     pub(crate) offset_x: f32,
//     pub(crate) offset_y: f32,
// }

// impl Offsets {
//     pub fn new(offset_x: f32, offset_y: f32) -> Self {
//         Self { offset_x, offset_y }
//     }
//     pub fn offset_point(&self, point: &Point) -> Point {
//         Point {
//             x: point.x + self.offset_x,
//             y: point.y + self.offset_y,
//         }
//     }
// }

// impl From<(f32, f32)> for Offsets {
//     fn from(value: (f32, f32)) -> Self {
//         Self {
//             offset_x: value.0,
//             offset_y: value.1,
//         }
//     }
// }
// impl From<raylib::math::Vector2> for Offsets {
//     fn from(value: Vector2) -> Self {
//         Self {
//             offset_x: value.x,
//             offset_y: value.y,
//         }
//     }
// }

#[derive(Debug, Clone, Copy)]
pub struct BBox {
    pub maximum: Point,
    pub minimum: Point,
}

impl BBox {
    pub fn new(maximum: impl Into<Point>, minimum: impl Into<Point>) -> Self {
        Self {
            maximum: maximum.into(),
            minimum: minimum.into(),
        }
    }

    pub fn width(&self) -> f32 {
        self.maximum.x - self.minimum.x
    }

    pub fn height(&self) -> f32 {
        self.maximum.y - self.minimum.y
    }
}

// impl Default for BBox {
//     fn default() -> Self {
//         Self {
//             minimum: Point { x: 0.0, y: 0.0 },
//             maximum: Point {
//                 x: crate::WIDTH as f32,
//                 y: crate::HEIGHT as f32,
//             },
//         }
//     }
// }

// impl From<(Point, Point)> for BBox {
//     fn from(value: (Point, Point)) -> Self {
//         Self {
//             minimum: value.0,
//             maximum: value.1,
//         }
//     }
// }

#[derive(Debug, Clone, Copy, Default)]
pub struct Margins {
    pub left: f32,
    pub right: f32,
    pub top: f32,
    pub bottom: f32,
}

impl Margins {
    #[inline]
    pub const fn all(v: f32) -> Self {
        Self {
            left: v,
            right: v,
            top: v,
            bottom: v,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub struct Viewport {
    x: f32,
    y: f32,
    width: f32,
    height: f32,
    margins: Margins,
}

impl Viewport {
    #[inline]
    pub const fn new(x: f32, y: f32, width: f32, height: f32) -> Self {
        Self {
            x,
            y,
            width,
            height,
            margins: Margins {
                left: 0.0,
                right: 0.0,
                top: 0.0,
                bottom: 0.0,
            },
        }
    }

    #[inline]
    pub const fn with_margins(mut self, margins: Margins) -> Self {
        self.margins = margins;
        self
    }

    /// Outer rectangle in screen coordinates.
    #[inline]
    pub fn outer_bbox(&self) -> BBox {
        BBox::new(
            (self.x + self.width, self.y + self.height),
            (self.x, self.y),
        )
    }

    /// Inner plotting area (after margins), in screen coordinates.
    #[inline]
    pub fn inner_bbox(&self) -> BBox {
        // minimum must be top-left; maximum must be bottom-right
        // let minimum = (self.x + self.margins.left, self.y + self.margins.top);
        // let maximum = (
        //     self.x + self.width - self.margins.right,
        //     self.y + self.height - self.margins.bottom,
        // );

        let minimum = (
            self.x + self.margins.left,
            self.y + self.height - self.margins.bottom,
        );
        let maximum = (
            self.x + self.width - self.margins.right,
            self.y + self.margins.top,
        );
        BBox::new(maximum, minimum)
    }
}

/// Linearly maps a value from one range to another
fn map_val(val: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    if (in_max - in_min).abs() < f32::EPSILON {
        return out_min; // Avoid division by zero if range is 0
    }
    (val - in_min) / (in_max - in_min) * (out_max - out_min) + out_min
}
#[derive(Debug, Clone, Copy)]
pub struct ViewTransformer {
    pub data_bounds: BBox,
    pub screen_bounds: Viewport,
}

impl ViewTransformer {
    pub fn new(data_bounds: BBox, screen_bounds: Viewport) -> Self {
        let sb = screen_bounds.inner_bbox();
        // debug_assert!(sb.minimum.x <= sb.maximum.x, "screen bbox has inverted X");
        // debug_assert!(sb.minimum.y <= sb.maximum.y, "screen bbox has inverted Y");

        Self {
            data_bounds,
            screen_bounds,
        }
    }

    /// Converts a point from Data Space to Screen Space
    pub fn to_screen(&self, point: &Point) -> Point {
        let screen_bounds = self.screen_bounds.inner_bbox();
        let x = map_val(
            point.x,
            self.data_bounds.minimum.x,
            self.data_bounds.maximum.x,
            screen_bounds.minimum.x,
            screen_bounds.maximum.x,
        );

        let y = map_val(
            point.y,
            self.data_bounds.minimum.y,
            self.data_bounds.maximum.y,
            screen_bounds.minimum.y,
            screen_bounds.maximum.y,
        );

        Point { x, y }
    }
}
