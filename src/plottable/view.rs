//! Bounding boxes, viewports, margins, and the data-to-screen transform.
//!
//! This module contains the spatial primitives that connect data coordinates
//! to screen pixels:
//!
//! * [`BBox<P>`] : a generic axis-aligned bounding box, parameterised over
//!   the point type. The type aliases [`DataBBox`] and [`ScreenBBox`]
//!   provide convenient specialisations.
//! * [`Viewport`] : defines a rectangular region on the screen together
//!   with inner [`Margins`], producing an outer bounding box (for the
//!   background / chrome) and an inner bounding box (for the data area).
//! * [`ViewTransformer`] : the core mapping that linearly projects
//!   [`Datapoint`]s to [`Screenpoint`]s, including y-axis inversion
//!   (data-space y grows up, screen-space y grows down).

use std::ops::Deref;

use raylib::math::Vector2;

use crate::plottable::point::{Datapoint, Screenpoint};

/// A generic axis-aligned bounding box over point type `P`.
///
/// `P` must dereference to [`Vector2`] and be constructible from one, which
/// is satisfied by both [`Datapoint`] and [`Screenpoint`].
///
/// The invariant `minimum.x <= maximum.x` and `minimum.y <= maximum.y` is
/// enforced by [`BBox::new`] (which re-orders the components) and
/// debug-asserted by [`BBox::from_min_max`].
#[derive(Debug, Clone, Copy)]
pub struct BBox<P> {
    /// Component-wise minimum corner.
    pub minimum: P,
    /// Component-wise maximum corner.
    pub maximum: P,
}

/// Bounding box in screen (pixel) coordinates.
pub type ScreenBBox = BBox<Screenpoint>;

/// Bounding box in data (world) coordinates.
pub type DataBBox = BBox<Datapoint>;

impl<P> BBox<P>
where
    P: Copy + Deref<Target = Vector2> + From<Vector2>,
{
    /// Create a bounding box from two arbitrary corners.
    ///
    /// The components are re-ordered so that `minimum` holds the smaller
    /// values and `maximum` holds the larger ones, regardless of the input
    /// order.
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
    /// Width of the bounding box (along the x-axis).
    pub fn width(&self) -> f32 {
        self.maximum.x - self.minimum.x
    }

    /// Height of the bounding box (along the y-axis).
    pub fn height(&self) -> f32 {
        self.maximum.y - self.minimum.y
    }
}

/// Pixel insets applied to a [`Viewport`] to separate the outer frame from
/// the inner data plotting area.
///
/// Use [`Margins::all`] for uniform insets or set each side individually.
#[derive(Debug, Clone, Copy, Default)]
pub struct Margins {
    /// Inset from the left edge in pixels.
    pub left: f32,
    /// Inset from the right edge in pixels.
    pub right: f32,
    /// Inset from the top edge in pixels.
    pub top: f32,
    /// Inset from the bottom edge in pixels.
    pub bottom: f32,
}

impl Margins {
    /// Create margins with the same inset on all four sides.
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

/// A rectangular screen region with optional inner margins.
///
/// The viewport defines where a graph is placed on the window. The outer
/// rectangle (`x`, `y`, `width`, `height`) covers the full area including
/// titles and labels, while the inner rectangle (after subtracting
/// [`Margins`]) is the data plotting area.
///
/// # Example
///
/// ```rust,ignore
/// let vp = Viewport::new(10.0, 10.0, 800.0, 600.0)
///     .with_margins(Margins { left: 60.0, right: 20.0, top: 50.0, bottom: 55.0 });
/// ```
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
            width: 800.0,
            height: 600.0,
            margins: Margins::default(),
        }
    }
}

impl Viewport {
    /// Create a viewport at `(x, y)` with the given dimensions and no margins.
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

    /// Set the inner margins, returning the modified viewport for chaining.
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

/// Linearly maps a scalar from one range to another.
///
/// Returns `out_min` when the input range is degenerate (zero width) to
/// avoid division by zero.
fn map_val(val: f32, in_min: f32, in_max: f32, out_min: f32, out_max: f32) -> f32 {
    if (in_max - in_min).abs() < f32::EPSILON {
        return out_min; // Avoid division by zero if range is 0
    }
    (val - in_min) / (in_max - in_min) * (out_max - out_min) + out_min
}
/// Transforms [`Datapoint`]s to [`Screenpoint`]s by linearly mapping the
/// data bounding box onto the screen bounding box.
///
/// The y-axis is **inverted** during the transform: data-space y increases
/// upward (mathematical convention) while screen-space y increases downward
/// (Raylib / window convention).
///
/// A `ViewTransformer` is normally constructed internally by
/// [`Graph::plot`](crate::graph::Graph) and passed to every
/// [`ChartElement::draw_in_view`](crate::plotter::ChartElement::draw_in_view)
/// call.
#[derive(Debug, Clone, Copy)]
pub struct ViewTransformer {
    /// The axis-aligned bounding box of the data in data coordinates.
    pub data_bounds: DataBBox,
    /// The viewport (with margins) that defines the screen target area.
    pub screen_bounds: Viewport,
}

impl ViewTransformer {
    /// Create a new transformer from explicit data and screen bounds.
    pub fn new(data_bounds: DataBBox, screen_bounds: Viewport) -> Self {
        Self {
            data_bounds,
            screen_bounds,
        }
    }

    /// Project a data-space point to screen-space coordinates.
    ///
    /// The x component is linearly mapped from the data range to the inner
    /// screen width. The y component is mapped with an inversion so that
    /// increasing data-y moves upward on the screen.
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
