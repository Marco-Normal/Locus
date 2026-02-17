#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
use crate::{
    colorscheme::{Colorscheme, Themable},
    plottable::{
        line::{Axis, GridLines, TickLabels},
        view::{ScreenBBox, ViewTransformer, Viewport},
    },
    plotter::{ChartElement, PlotElement},
};
use derive_builder::Builder;
use raylib::prelude::RaylibScissorModeExt;
/// Represents a graph over `subject`, orchestrating elements such as axis, gridlines
/// and other important pieces.
pub struct Graph<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default,
{
    subject: T,
}

impl<T> Graph<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default,
{
    pub fn new(subject: T) -> Self {
        Self { subject }
    }
}

#[derive(Debug, Clone)]
pub struct ConfiguredElement<E>
where
    E: ChartElement,
{
    pub(crate) element: E,
    pub(crate) configs: E::Config,
}

impl<E> ConfiguredElement<E>
where
    E: ChartElement,
    <E as ChartElement>::Config: Default + Themable,
{
    pub fn new(element: E, configs: E::Config) -> Self {
        Self { element, configs }
    }
    pub fn draw(&self, rl: &mut raylib::prelude::RaylibDrawHandle, view: &ViewTransformer) {
        self.element.draw_in_view(rl, &self.configs, view);
    }
}

impl<E: ChartElement> Themable for ConfiguredElement<E>
where
    E::Config: Themable,
{
    fn apply_theme(&mut self, scheme: &Colorscheme) {
        self.configs.apply_theme(scheme);
    }
}

impl<E: ChartElement> ConfiguredElement<E>
where
    E::Config: Default,
{
    /// Create with default configuration.
    pub fn with_defaults(element: E) -> Self {
        Self {
            element,
            configs: E::Config::default(),
        }
    }
}

impl<E: ChartElement> ConfiguredElement<E> {
    /// Modify the config via a closure, returning self for chaining.
    #[must_use]
    pub fn configure(mut self, f: impl FnOnce(&mut E::Config)) -> Self {
        f(&mut self.configs);
        self
    }
}

/// Main configuration for the graph. It's possible to pass configuration from the
/// `subject`, as well as, for axis, gridlines, offset for the graph, and bounding box.
#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned", name = "GraphBuilder", build_fn(skip))]
pub struct GraphConfig<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default + Themable,
{
    #[builder(default)]
    subject_configs: T::Config,
    #[builder(default)]
    viewport: Viewport,
    #[builder(setter(into, strip_option), default = "None")]
    axis: Option<ConfiguredElement<Axis>>,
    #[builder(setter(into, strip_option), default = "None")]
    grid: Option<ConfiguredElement<GridLines>>,
    #[builder(default)]
    colorscheme: Colorscheme,
    #[builder(setter(into, strip_option), default = "None")]
    ticks: Option<ConfiguredElement<TickLabels>>,
}

impl<T> GraphBuilder<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default + Themable,
{
    #[allow(clippy::missing_errors_doc)]
    pub fn build(self) -> Result<GraphConfig<T>, GraphBuilderError> {
        Ok(GraphConfig {
            subject_configs: self.subject_configs.unwrap_or_default(),
            viewport: self.viewport.unwrap_or_default(),
            axis: self.axis.unwrap_or(None),
            grid: self.grid.unwrap_or(None),
            colorscheme: self.colorscheme.unwrap_or_default(),
            ticks: self.ticks.unwrap_or(None),
        }
        .resolve_theme())
    }
}
impl<T> GraphConfig<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default + Themable,
{
    /// Resolves theme-driven defaults (subject + axis/grid configs) once.
    /// Call this after `build()` and reuse the returned config across frames.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn resolve_theme(mut self) -> Self {
        if let Some(axis) = &mut self.axis {
            axis.apply_theme(&self.colorscheme);
        }
        if let Some(grid) = &mut self.grid {
            grid.apply_theme(&self.colorscheme);
        }
        if let Some(ticks) = &mut self.ticks {
            ticks.apply_theme(&self.colorscheme);
        }
        self.subject_configs.apply_theme(&self.colorscheme);
        self
    }
}

impl<T: ChartElement> PlotElement for Graph<T>
where
    <T as ChartElement>::Config: Default + Themable,
{
    type Config = GraphConfig<T>;

    fn plot(&self, rl: &mut raylib::prelude::RaylibDrawHandle, configs: &GraphConfig<T>) {
        // We need to construct the view where the graph elements will live.
        // As such, we need to provide the screen-bounds, given by the configs
        // and the data-bounds, given by the `subject.data_bounds()`
        // rl.clear_background(configs.colorscheme.background);
        let screen = configs.viewport;
        let data_bbox = if let Some(axis) = &configs.axis {
            axis.element.data_bounds()
        } else {
            self.subject.data_bounds()
        };
        let inner = screen.inner_bbox();
        let inner_viewport = Viewport::new(
            inner.minimum.x,
            inner.minimum.y,
            inner.width(),
            inner.height(),
        );
        let view = ViewTransformer::new(data_bbox, inner_viewport);
        {
            let inner_bbox = screen.inner_bbox();
            let (x, y, w, h) = scissor_rect_from_bbox(inner_bbox);
            let mut scissors = rl.begin_scissor_mode(x, y, w, h);
            // We have all the necessary parts for constructing the graph. With that is a job of
            // seeing what we have and what don't.
            if let Some(grid) = &configs.grid {
                grid.draw(&mut scissors, &view);
            }

            // We plot the subject inside the view.
            // configs.subject_configs.apply_theme(&configs.colorscheme);
            self.subject
                .draw_in_view(&mut scissors, &configs.subject_configs, &view);
        }
        // NOTE: Axis shouldn't be scissored, neither the ticks;
        if let Some(axis) = &configs.axis {
            axis.draw(rl, &view);
        }
        if let Some(ticks) = &configs.ticks {
            ticks.draw(rl, &view);
        }
    }
}
#[allow(clippy::cast_possible_truncation)]
fn scissor_rect_from_bbox(b: ScreenBBox) -> (i32, i32, i32, i32) {
    // Round to pixel grid; clamp sizes to >= 0
    let x = b.minimum.x.round() as i32;
    let y = b.minimum.y.round() as i32;
    let width = b.width().round().max(0.0) as i32;
    let height = b.height().round().max(0.0) as i32;
    (x, y, width, height)
}
