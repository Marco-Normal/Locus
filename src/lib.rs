#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
//! # Locus
//!
//! **Locus** is a composable graphing and plotting library for Rust, built on
//! top of [raylib](https://crates.io/crates/raylib). It provides a structured,
//! trait-driven architecture for visualizing 2D data in real-time windows.
//!
//! The library separates concerns into two layers:
//!
//! * **Data-space elements** ([`ChartElement`](plotter::ChartElement)) that live
//!   in arbitrary coordinate systems and are projected onto the screen through a
//!   [`ViewTransformer`](plottable::view::ViewTransformer).
//! * **Screen-space elements** ([`PlotElement`](plotter::PlotElement)) that are
//!   rendered directly in pixel coordinates.
//!
//! A [`Graph`](graph::Graph) wraps any `ChartElement` and orchestrates axes,
//! grid lines, tick marks, labels, legends, annotations, and color themes so
//! that the caller only needs to supply data and configuration.
//!
//! # Quick start
//!
//! ```rust,no_run
//! use locus::prelude::*;
//! use raylib::prelude::*;
//! const IMAGE_SIZE: i32 = 90;
//! const WIDTH: i32 = 16 * IMAGE_SIZE;
//! const HEIGHT: i32 = 9 * IMAGE_SIZE;
//! let (mut rl, thread) = raylib::init()
//!     .width(WIDTH)
//!     .height(HEIGHT)
//!     .title("My Plot")
//!     .build();
//!
//! let data = Dataset::new(vec![(1.0, 2.0), (3.0, 4.0), (5.0, 1.0)]);
//! let scatter = ScatterPlot::new(&data);
//! let graph = Graph::new(scatter);
//! let scheme = GITHUB_DARK.clone();
//! let axis = Axis::fitting(
//!     data.range_min.x..data.range_max.x,
//!     data.range_min.y..data.range_max.y,
//!     0.05,
//!     10,
//! );
//!
//! while !rl.window_should_close() {
//!     let mut d = rl.begin_drawing(&thread);
//!     d.clear_background(scheme.background);
//!     graph.plot(
//!         &mut d,
//!         &GraphBuilder::default()
//!             .viewport(
//!                 Viewport::new(0.0, 0.0, WIDTH as f32, HEIGHT as f32)
//!                     .with_margins(Margins::all(50.0)),
//!             )
//!             .colorscheme(scheme.clone())
//!             .axis(ConfiguredElement::with_defaults(axis))
//!             .grid(ConfiguredElement::with_defaults(
//!                 GridLines::new(axis, Orientation::default()),
//!             ))
//!             .ticks(ConfiguredElement::with_defaults(TickLabels::new(axis)))
//!             .title("My Scatter Plot")
//!             .xlabel("X")
//!             .ylabel("Y")
//!             .build()
//!             .unwrap(),
//!     );
//! }
//! ```
//!
//! # Modules
//!
//! | Module | Purpose |
//! |---|---|
//! | [`colorscheme`] | Predefined color themes and the [`Themable`](colorscheme::Themable) trait |
//! | [`dataset`] | The [`Dataset`](dataset::Dataset) container for collections of data points |
//! | [`graph`] | The [`Graph`](graph::Graph) orchestrator and its builder |
//! | [`plottable`] | Primitive visual elements: points, lines, scatter plots, text, ticks, legends, annotations, and the view transform |
//! | [`plotter`] | Core rendering traits ([`PlotElement`](plotter::PlotElement), [`ChartElement`](plotter::ChartElement)) |
//!
//! # Feature highlights
//!
//! * Builder-driven configuration for every visual element.
//! * Automatic "nice number" axis snapping and tick generation (linear,
//!   logarithmic, and symmetric-log scales).
//! * Multiple built-in color schemes (Dracula, Nord, Viridis, Solarized,
//!   GitHub, Matplotlib).
//! * Per-point dynamic size, color, and shape mapping on scatter plots.
//! * Data-space annotations with optional leader arrows.
//! * Legends with configurable position, indicator shapes, and styling.

pub mod colorscheme;
pub mod dataset;
pub mod graph;
pub mod plottable;
pub mod plotter;

pub use plottable::annotation::{Annotation, AnnotationPosition};
pub use plottable::legend::{Legend, LegendEntry, LegendPosition};
pub use plottable::text::{Anchor, FontHandle, HAlign, TextLabel, TextStyle, VAlign};

pub mod prelude {
    pub use super::colorscheme::*;
    pub use super::dataset::*;
    pub use super::graph::*;
    pub use super::plottable::annotation::*;
    pub use super::plottable::legend::*;
    pub use super::plottable::line::*;
    pub use super::plottable::point::*;
    pub use super::plottable::scatter::*;
    pub use super::plottable::text::*;
    pub use super::plottable::ticks::*;
    pub use super::plottable::view::*;
    pub use super::plotter::*;
}
