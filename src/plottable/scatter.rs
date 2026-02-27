//! Scatter plot element with per-point attribute mapping.
//!
//! [`ScatterPlot`] renders a [`Dataset`] as individual points inside a
//! [`ViewTransformer`]. Each visual attribute (size, color, shape) can be
//! set to a fixed value or mapped dynamically per-point via a closure,
//! enabling techniques like cluster coloring or bubble charts.
//!
//! # Example
//!
//! ```rust
//! use locus::prelude::*;
//! use raylib::color::Color;
//! # let dataset = Dataset::new(vec![(0.0,0.0), (1.0,1.0), (2.0, 2.0)]);
//! let scatter = ScatterPlot::new(&dataset);
//! let config = ScatterPlotBuilder::default()
//!     .fixed_color(Color::RED)
//!     .fixed_size(4.0)
//!     .build()
//!     .unwrap();
//! ```

use crate::{
    colorscheme::Themable,
    dataset::Dataset,
    plottable::{
        point::{Datapoint, PointConfigBuilder, Shape},
        view::{DataBBox, ViewTransformer},
    },
    plotter::{ChartElement, PlotElement},
};
use derive_builder::Builder;
use raylib::prelude::Color;

/// A closure that computes point size from the data point and its index.
pub type DynamicSize = Box<dyn Fn(&Datapoint, usize) -> f32>;
/// A closure that computes point color from the data point and its index.
pub type DynamicColor = Box<dyn Fn(&Datapoint, usize) -> Color>;
/// A closure that computes point shape from the data point and its index.
pub type DynamicShape = Box<dyn Fn(&Datapoint, usize) -> Shape>;
/// Generic per-point attribute mapping closure.
pub type Dynamic<T> = Box<dyn Fn(&Datapoint, usize) -> T>;

/// Determines whether a visual attribute is a single constant or varies
/// per data point.
pub enum Strategy<T> {
    /// The same value is used for every point.
    Fixed(T),
    /// A closure is called for each point to compute the value.
    Dynamic(Dynamic<T>),
}

/// Configuration for a [`ScatterPlot`].
///
/// Each visual property (size, color, shape) is optional. When `None`,
/// sensible defaults are used (size = 5, shape = circle, color resolved
/// from the theme cycle). Properties can be set to a [`Strategy::Fixed`]
/// constant or a [`Strategy::Dynamic`] closure for per-point variation.
///
/// Construct via [`ScatterPlotBuilder`]:
///
/// ```rust
/// use locus::prelude::*;
/// use raylib::color::Color;
/// ScatterPlotBuilder::default()
///     .fixed_color(Color::BLUE)
///     .mapped_size(Box::new(|pt, _i| pt.y.abs()))
///     .build()
///     .unwrap();
/// ```
#[derive(Builder)]
#[builder(pattern = "owned", name = "ScatterPlotBuilder")]
pub struct ScatterPlotConfig {
    /// Point size strategy. `None` falls back to a default of 5 pixels.
    #[builder(setter(into, strip_option), default = "None")]
    size: Option<Strategy<f32>>,
    /// Point color strategy. `None` is resolved from the color scheme.
    #[builder(setter(into, strip_option), default = "None")]
    color: Option<Strategy<Color>>,
    /// Point shape strategy. `None` falls back to [`Shape::Circle`].
    #[builder(setter(into, strip_option), default = "None")]
    shape: Option<Strategy<Shape>>,
}

impl Default for ScatterPlotConfig {
    fn default() -> Self {
        ScatterPlotBuilder::default()
            .build()
            .expect("Will never fail")
    }
}

impl ScatterPlotBuilder {
    /// Use a constant point size for every data point.
    #[must_use]
    pub fn fixed_size(self, size: f32) -> Self {
        Self {
            size: Some(Some(Strategy::Fixed(size))),
            ..self
        }
    }
    /// Use a constant color for every data point.
    #[must_use]
    pub fn fixed_color(self, color: Color) -> Self {
        Self {
            color: Some(Some(Strategy::Fixed(color))),
            ..self
        }
    }

    /// Use a constant shape for every data point.
    #[must_use]
    pub fn fixed_shape(self, shape: Shape) -> Self {
        Self {
            shape: Some(Some(Strategy::Fixed(shape))),
            ..self
        }
    }

    /// Compute point color dynamically from each data point and its index.
    #[must_use]
    pub fn mapped_color(self, color_func: DynamicColor) -> Self {
        Self {
            color: Some(Some(Strategy::Dynamic(color_func))),
            ..self
        }
    }

    /// Compute point shape dynamically from each data point and its index.
    #[must_use]
    pub fn mapped_shape(self, shape_func: DynamicShape) -> Self {
        Self {
            shape: Some(Some(Strategy::Dynamic(shape_func))),
            ..self
        }
    }

    /// Compute point size dynamically from each data point and its index.
    #[must_use]
    pub fn mapped_size(self, size_func: DynamicSize) -> Self {
        Self {
            size: Some(Some(Strategy::Dynamic(size_func))),
            ..self
        }
    }
}

/// A scatter plot that renders every point in a [`Dataset`] as an
/// individual marker inside a view transform.
///
/// Each point is projected from data space to screen space via the
/// [`ViewTransformer`], then drawn as a [`Screenpoint`](crate::plottable::point::Screenpoint)
/// with attributes determined by the [`ScatterPlotConfig`].
pub struct ScatterPlot<'a> {
    /// Reference to the dataset being visualized.
    pub data: &'a Dataset,
}

impl<'a> ScatterPlot<'a> {
    /// Create a scatter plot over the given dataset.
    #[must_use]
    pub fn new(data: &'a Dataset) -> Self {
        Self { data }
    }
}

impl ChartElement for ScatterPlot<'_> {
    type Config = ScatterPlotConfig;

    fn draw_in_view(
        &self,
        rl: &mut raylib::prelude::RaylibDrawHandle,
        configs: &ScatterPlotConfig,
        view: &ViewTransformer,
    ) {
        self.data.data.iter().enumerate().for_each(|(i, p)| {
            let screen_point = view.to_screen(p);
            let size = match &configs.size {
                Some(strat) => match strat {
                    Strategy::Fixed(c) => *c,
                    Strategy::Dynamic(func) => func(p, i),
                },
                None => 5.0,
            };

            let shape = match &configs.shape {
                Some(strat) => match strat {
                    Strategy::Fixed(s) => *s,
                    Strategy::Dynamic(func) => func(p, i),
                },
                None => Shape::Circle,
            };
            let color = match &configs.color {
                Some(strat) => match strat {
                    Strategy::Fixed(c) => *c,
                    Strategy::Dynamic(func) => func(p, i),
                },
                None => Color::BLACK,
            };
            screen_point.plot(
                rl,
                &PointConfigBuilder::default()
                    .size(size)
                    .shape(shape)
                    .color(color)
                    .build()
                    .expect("Failed to build point config"),
            );
        });
    }

    fn data_bounds(&self) -> DataBBox {
        DataBBox {
            minimum: Datapoint((self.data.range_min.x, self.data.range_min.y).into()),
            maximum: Datapoint((self.data.range_max.x, self.data.range_max.y).into()),
        }
    }
}

impl Themable for ScatterPlotConfig {
    fn apply_theme(&mut self, scheme: &crate::colorscheme::Colorscheme) {
        match &self.color {
            Some(_) => (),
            None => {
                self.color = Some(Strategy::Fixed(
                    scheme.cycle.first().copied().unwrap_or(Color::BLACK),
                ));
            }
        }
    }
}
