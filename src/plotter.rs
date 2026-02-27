//! Core rendering traits that define how visual elements are drawn.
//!
//! Locus distinguishes between two kinds of drawable elements:
//!
//! * [`PlotElement`] for items that live in **screen space** and can be drawn
//!   directly with pixel coordinates (e.g. text labels, the fully composed
//!   [`Graph`](crate::graph::Graph) itself).
//! * [`ChartElement`] for items that live in **data space** and require a
//!   [`ViewTransformer`] to project
//!   their coordinates onto the screen (e.g. scatter plots, axes, grid lines).
//!
//! Every concrete visual primitive in the [`plottable`](crate::plottable)
//! module implements one (or both) of these traits. The associated `Config`
//! type carries all style and layout parameters needed to render the element.

use raylib::prelude::RaylibDrawHandle;

use crate::plottable::view::{DataBBox, ViewTransformer};

/// A drawable element that operates entirely in screen (pixel) coordinates.
///
/// Types implementing `PlotElement` do not need a coordinate transform; they
/// are rendered at the exact positions stored in their fields. The canonical
/// example is [`TextLabel`](crate::plottable::text::TextLabel), but
/// [`Graph`](crate::graph::Graph) also implements this trait so that the
/// fully assembled graph can be drawn with a single `.plot()` call.
///
/// The associated [`Config`](PlotElement::Config) type carries visual
/// configuration (colors, sizes, styles) that is passed by reference at
/// draw time, allowing the same element to be rendered with different
/// settings without mutation.
pub trait PlotElement {
    /// Configuration type that controls how this element looks.
    type Config;

    /// Render the element onto `rl` using the given `configs`.
    fn plot(&self, rl: &mut RaylibDrawHandle, configs: &Self::Config);
}

/// A drawable element that lives in an arbitrary data coordinate system.
///
/// Types implementing `ChartElement` express their geometry in data units
/// (e.g. real-valued x/y) and rely on a
/// [`ViewTransformer`] to map those
/// coordinates to screen pixels at draw time.
///
/// Implementors must also report their spatial extent via
/// [`data_bounds`](ChartElement::data_bounds) so that the
/// [`Graph`](crate::graph::Graph) can compute an appropriate view transform
/// that fits all elements within the viewport.
pub trait ChartElement {
    /// Configuration type that controls how this element looks.
    type Config;

    /// Render the element, using `view` to project data coordinates to screen
    /// coordinates.
    fn draw_in_view(
        &self,
        rl: &mut RaylibDrawHandle,
        configs: &Self::Config,
        view: &ViewTransformer,
    );

    /// Return the axis-aligned bounding box of this element in data
    /// coordinates.
    fn data_bounds(&self) -> DataBBox;
}
