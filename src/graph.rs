#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
use crate::{
    colorscheme::{Colorscheme, Themable},
    plottable::{
        line::{
            Axis, AxisConfigs, AxisConfigsBuilder, GridLines, GridLinesConfig,
            GridLinesConfigBuilder,
        },
        view::{ViewTransformer, Viewport},
    },
    plotter::{ChartElement, PlotElement},
};
use derive_builder::Builder;
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
/// Main configuration for the graph. It's possible to pass configuration from the
/// `subject`, as well as, for axis, gridlines, offset for the graph, and bounding box.
#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned", name = "GraphBuilder")]
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
    axis: Option<Axis>,
    #[builder(setter(into, strip_option), default = "None")]
    axis_configs: Option<AxisConfigs>,
    #[builder(setter(into, strip_option), default = "None")]
    grid: Option<GridLines>,
    #[builder(setter(into, strip_option), default = "None")]
    grid_configs: Option<GridLinesConfig>,
    #[builder(default)]
    colorscheme: Colorscheme,
}

impl<T> GraphConfig<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default + Themable,
{
    /// Resolves theme-driven defaults (subject + axis/grid configs) once.
    /// Call this after `build()` and reuse the returned config across frames.
    #[must_use]
    pub fn resolve_theme(mut self) -> Self {
        // Subject config: theme-driven defaults (e.g. ScatterPlot default color)
        self.subject_configs.apply_theme(&self.colorscheme);
        // Grid config: if enabled and missing config, create a themed default; otherwise theme it.
        if self.grid.is_some() {
            match &mut self.grid_configs {
                Some(cfg) => cfg.apply_theme(&self.colorscheme),
                None => {
                    self.grid_configs = Some(
                        GridLinesConfigBuilder::default()
                            .color(self.colorscheme.grid)
                            .build()
                            .expect("Default values set"),
                    );
                }
            }
        }
        // Axis config: if enabled and missing config, create a themed default; otherwise theme it.
        if self.axis.is_some() {
            match &mut self.axis_configs {
                Some(cfg) => cfg.apply_theme(&self.colorscheme),
                None => {
                    self.axis_configs = Some(
                        AxisConfigsBuilder::default()
                            .color(self.colorscheme.axis)
                            .build()
                            .expect("Default values set"),
                    );
                }
            }
        }
        self
    }
}

impl<T: ChartElement> PlotElement for Graph<T>
where
    <T as ChartElement>::Config: Default + Themable,
{
    type Config = GraphConfig<T>;

    fn plot(&self, rl: &mut raylib::prelude::RaylibDrawHandle, mut configs: &GraphConfig<T>) {
        // We need to construct the view where the graph elements will live.
        // As such, we need to provide the screen-bounds, given by the configs
        // and the data-bounds, given by the `subject.data_bounds()`
        // rl.clear_background(configs.colorscheme.background);
        let screen = configs.viewport;
        let data_bbox = if let Some(axis) = configs.axis {
            axis.data_bounds()
        } else {
            self.subject.data_bounds()
        };
        let view = ViewTransformer::new(data_bbox, screen);
        // We have all the necessary parts for constructing the graph. With that is a job of
        // seeing what we have and what don't.
        if let Some(grid) = configs.grid {
            // If the grid has a config, use it, else, defaults to configs from the graph + defaults
            // from element.
            // NOTE: Should always unwrap with a configuration, if the user applied theming.
            let grid_conf = configs.grid_configs.unwrap_or({
                GridLinesConfigBuilder::default()
                    .color(configs.colorscheme.grid)
                    .build()
                    .expect("Default values set")
            });
            grid.draw_in_view(rl, &grid_conf, &view);
        }

        // We plot the subject inside the view.
        // configs.subject_configs.apply_theme(&configs.colorscheme);
        self.subject
            .draw_in_view(rl, &configs.subject_configs, &view);
        // Plot the axis and the story is the same. If we have an config given by the user
        // use it, else, defaults to graph configs + element defaults.
        if let Some(axis) = configs.axis {
            let axis_conf = configs.axis_configs.unwrap_or(
                AxisConfigsBuilder::default()
                    .color(configs.colorscheme.axis)
                    .build()
                    .expect("Default values set"),
            );
            axis.draw_in_view(rl, &axis_conf, &view);
        }
    }
}
