//! Fundamental point types and shape primitives.
//!
//! Locus uses two distinct point wrappers to enforce a clear separation
//! between data-space and screen-space coordinates at the type level:
//!
//! * [`Datapoint`] represents a position in the user's data coordinate system.
//! * [`Screenpoint`] represents a position in pixel (screen) coordinates.
//!
//! Both are newtypes over [`Vector2`], implement
//! [`Deref`](std::ops::Deref) for ergonomic field access, and offer
//! [`From`] conversions from `(f32, f32)` tuples and `Vector2` values.
//!
//! [`Screenpoint`] additionally implements [`PlotElement`] so that individual
//! points can be rendered with a configurable [`Shape`], size, and color.

use crate::plotter::PlotElement;
use derive_builder::Builder;
use raylib::math::Vector2;
use raylib::prelude::*;

/// A point in data (world) coordinates.
///
/// `Datapoint` is a transparent wrapper around [`Vector2`]. It dereferences
/// to `Vector2` so that `.x` and `.y` are directly accessible, while
/// remaining a distinct type from [`Screenpoint`] to prevent accidental
/// mixing of coordinate systems.
///
/// # Construction
///
/// ```rust
/// use locus::prelude::*;
/// use raylib::math::Vector2;
/// let p = Datapoint::new(3.0, 4.5);
/// let p: Datapoint = (3.0, 4.5).into();
/// let p: Datapoint = Vector2::new(3.0, 4.5).into();
/// ```
#[derive(Clone, Copy, Debug)]
pub struct Datapoint(pub Vector2);

impl Datapoint {
    /// Create a new data-space point from explicit coordinates.
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self((x, y).into())
    }
}

impl From<Vector2> for Datapoint {
    fn from(value: Vector2) -> Self {
        Self(value)
    }
}

impl From<&Vector2> for Datapoint {
    fn from(value: &Vector2) -> Self {
        Self(*value)
    }
}

impl From<(f32, f32)> for Datapoint {
    fn from(value: (f32, f32)) -> Self {
        Datapoint(value.into())
    }
}

impl std::ops::DerefMut for Datapoint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for Datapoint {
    type Target = Vector2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
/// A point in screen (pixel) coordinates.
///
/// `Screenpoint` is a transparent wrapper around [`Vector2`], analogous to
/// [`Datapoint`] but representing a position on the rendered window rather
/// than in the user's data space.
///
/// Implements [`PlotElement`] so that a single point can be drawn as a
/// circle, triangle, or rectangle via [`PointConfig`].
#[derive(Clone, Copy, Debug)]
pub struct Screenpoint(pub Vector2);

impl Screenpoint {
    /// Create a new screen-space point from pixel coordinates.
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self((x, y).into())
    }
}
impl From<Vector2> for Screenpoint {
    fn from(value: Vector2) -> Self {
        Self(value)
    }
}

impl From<&Vector2> for Screenpoint {
    fn from(value: &Vector2) -> Self {
        Self(*value)
    }
}

impl From<(f32, f32)> for Screenpoint {
    fn from(value: (f32, f32)) -> Self {
        Screenpoint(value.into())
    }
}

impl std::ops::DerefMut for Screenpoint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for Screenpoint {
    type Target = Vector2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

/// Geometric shape used to render a point or legend swatch.
#[derive(Debug, Clone, Copy)]
pub enum Shape {
    /// Filled circle (default).
    Circle,
    /// Filled equilateral triangle.
    Triangle,
    /// Filled axis-aligned rectangle.
    Rectangle,
}

/// Visual configuration for drawing a single [`Screenpoint`].
///
/// Built via [`PointConfigBuilder`]:
///
/// ```rust
/// use locus::prelude::*;
/// use raylib::color::Color;
/// let cfg = PointConfigBuilder::default()
///     .color(Color::RED)
///     .size(6.0)
///     .shape(Shape::Triangle)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Builder)]
#[builder(pattern = "owned")]
#[builder(default)]
pub struct PointConfig {
    /// Fill color of the point.
    color: Color,
    /// Radius (for circles) or half-extent (for other shapes) in pixels.
    size: f32,
    /// Geometric shape used to render the point.
    shape: Shape,
}

impl Default for PointConfig {
    fn default() -> Self {
        Self {
            color: Color::RED,
            size: 10.0,
            shape: Shape::Circle,
        }
    }
}

impl PlotElement for Screenpoint {
    type Config = PointConfig;
    #[allow(clippy::cast_possible_truncation)]
    fn plot(&self, rl: &mut raylib::prelude::RaylibDrawHandle, configs: &PointConfig) {
        let x = self.x;
        let y = self.y;
        match configs.shape {
            Shape::Circle => {
                rl.draw_circle(x as i32, y as i32, configs.size, configs.color);
            }
            Shape::Triangle => {
                rl.draw_triangle(
                    Vector2::new(
                        x + configs.size * f32::cos(330.0_f32.to_radians()),
                        y + configs.size * f32::sin(330.0_f32.to_radians()),
                    ),
                    Vector2::new(
                        x + configs.size * f32::cos(210.0_f32.to_radians()),
                        y + configs.size * f32::sin(210.0_f32.to_radians()),
                    ),
                    Vector2::new(x, y + configs.size),
                    configs.color,
                );
            }
            Shape::Rectangle => {
                rl.draw_rectangle_v(
                    Vector2::new(x, y),
                    Vector2::new(configs.size, configs.size),
                    configs.color,
                );
            }
        }
    }
}
