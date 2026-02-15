use std::ops::Deref;

use raylib::math::Vector2;

use crate::{
    HEIGHT, WIDTH,
    plottable::point::{Datapoint, Screenpoint},
};

#[derive(Debug, Clone, Copy)]
pub struct BBox<P> {
    pub minimum: P,
    pub maximum: P,
}

pub type ScreenBBox = BBox<Screenpoint>;
pub type DataBBox = BBox<Datapoint>;

impl<P> BBox<P>
where
    P: Copy + Deref<Target = Vector2> + From<Vector2>,
{
    pub fn new(a: impl Into<P>, b: impl Into<P>) -> Self {
        let a: P = a.into();
        let b: P = b.into();
        let min_x = a.x.min(b.x);
        let min_y = a.y.min(b.y);
        let max_x = a.x.max(b.x);
        let max_y = a.y.max(b.y);
        Self {
            minimum: Vector2::new(min_x, min_y).into(),
            maximum: Vector2::new(max_x, max_y).into(),
        }
    }
    /// Creates a bounding box assuming `minimum` and `maximum` already satisfy invariants.
    /// Debug-asserts the invariant to catch mistakes early.
    #[must_use]
    pub fn from_min_max(minimum: impl Into<P>, maximum: impl Into<P>) -> Self {
        let minimum: P = minimum.into();
        let maximum: P = maximum.into();
        debug_assert!(
            minimum.x <= maximum.x,
            "BBox invariant violated: min.x > max.x"
        );
        debug_assert!(
            minimum.y <= maximum.y,
            "BBox invariant violated: min.y > max.y"
        );
        Self { minimum, maximum }
    }
    pub fn width(&self) -> f32 {
        self.maximum.x - self.minimum.x
    }

    pub fn height(&self) -> f32 {
        self.maximum.y - self.minimum.y
    }
}

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

#[derive(Debug, Clone, Copy)]
pub struct Viewport {
    pub(crate) x: f32,
    pub(crate) y: f32,
    pub(crate) width: f32,
    pub(crate) height: f32,
    margins: Margins,
}

impl Default for Viewport {
    fn default() -> Self {
        Self {
            x: 0.0,
            y: 0.0,
            width: WIDTH as f32,
            height: HEIGHT as f32,
            margins: Margins::default(),
        }
    }
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
    /// NOTE: this returns a *numeric* bounding box where `minimum.y <= maximum.y`.
    /// In Raylib screen space that means:
    /// - `minimum` is the top-left corner
    /// - `maximum` is the bottom-right corner
    #[inline]
    pub fn outer_bbox(&self) -> ScreenBBox {
        let minimum = (self.x, self.y);
        let maximum = (self.x + self.width, self.y + self.height);
        BBox::new(minimum, maximum)
    }

    /// Inner plotting area (after margins), in screen coordinates.
    ///
    /// Note: this returns a *numeric* bounding box where `minimum.y <= maximum.y`.
    /// In Raylib screen space that means:
    /// - `minimum` is the top-left corner
    /// - `maximum` is the bottom-right corner
    #[inline]
    pub fn inner_bbox(&self) -> ScreenBBox {
        let minimum = (self.x + self.margins.left, self.y + self.margins.top);
        let maximum = (
            self.x + self.width - self.margins.right,
            self.y + self.height - self.margins.bottom,
        );
        BBox::new(minimum, maximum)
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
    pub data_bounds: DataBBox,
    pub screen_bounds: Viewport,
}

impl ViewTransformer {
    pub fn new(data_bounds: DataBBox, screen_bounds: Viewport) -> Self {
        Self {
            data_bounds,
            screen_bounds,
        }
    }

    /// Converts a point from Data Space to Screen Space
    pub fn to_screen(&self, point: &Datapoint) -> Screenpoint {
        let screen_bounds = self.screen_bounds.inner_bbox();
        let x = map_val(
            point.x,
            self.data_bounds.minimum.x,
            self.data_bounds.maximum.x,
            screen_bounds.minimum.x,
            screen_bounds.maximum.x,
        );

        // Explicit Y inversion:
        // data min (bottom) -> screen max (bottom)
        // data max (top)    -> screen min (top)
        let y = map_val(
            point.y,
            self.data_bounds.minimum.y,
            self.data_bounds.maximum.y,
            screen_bounds.maximum.y,
            screen_bounds.minimum.y,
        );

        Screenpoint((x, y).into())
    }
}
#[cfg(test)]
mod tests {
    use super::*;

    fn assert_approx(a: f32, b: f32) {
        assert!((a - b).abs() < 1e-5, "expected {b}, got {a}");
    }

    #[test]
    fn to_screen_flips_y_cartesian_to_raylib() {
        let data = BBox::new((0.0, 0.0), (10.0, 10.0));
        let viewport = Viewport::new(0.0, 0.0, 100.0, 100.0);
        let view = ViewTransformer::new(data, viewport);

        // data bottom-left -> screen bottom-left
        let p = view.to_screen(&Datapoint::new(0.0, 0.0));
        assert_approx(p.x, 0.0);
        assert_approx(p.y, 100.0);

        // data top-left -> screen top-left
        let p = view.to_screen(&Datapoint::new(0.0, 10.0));
        assert_approx(p.x, 0.0);
        assert_approx(p.y, 0.0);

        // data bottom-right -> screen bottom-right
        let p = view.to_screen(&Datapoint::new(10.0, 0.0));
        assert_approx(p.x, 100.0);
        assert_approx(p.y, 100.0);
    }
}
