#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
use raylib::prelude::RaylibDrawHandle;

use crate::plottable::view::{BBox, ViewTransformer};

/// Main trait for primitive types, where there is no need for a reference bounded by
/// an axis or scale. Elements implementing this trait should be plotted in the same
/// coordinates that they are in, with a possible offset.
pub trait PlotElement {
    type Config;
    fn plot(&self, rl: &mut RaylibDrawHandle, configs: &Self::Config);
}

/// Main trait for elements that require some sort of translation between arbitrary coordinates
/// and screen coordinates, where `view` serves the purpose to make that translation.
pub trait ChartElement {
    type Config;
    fn draw_in_view(
        &self,
        rl: &mut RaylibDrawHandle,
        configs: &Self::Config,
        view: &ViewTransformer,
    );
    fn data_bounds(&self) -> BBox;
}
