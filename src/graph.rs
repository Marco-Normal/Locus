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
        view::{BBox, Offsets, ViewTransformer},
    },
    plotter::{ChartElement, PlotElement},
};
use derive_builder::Builder;
use raylib::prelude::RaylibDraw;
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
    offset: Offsets,
    #[builder(default)]
    bounding_box: BBox,
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

impl<T: ChartElement> PlotElement for Graph<T>
where
    <T as ChartElement>::Config: Default + Themable,
{
    type Config = GraphConfig<T>;

    fn plot(&self, rl: &mut raylib::prelude::RaylibDrawHandle, mut configs: Self::Config) {
        // We need to construct the view where the graph elements will live.
        // As such, we need to provide the screen-bounds, given by the configs
        // and the data-bounds, given by the `subject.data_bounds()`
        rl.clear_background(configs.colorscheme.background);
        let screen_bbox = configs.bounding_box;
        let data_bbox = if let Some(axis) = configs.axis {
            axis.data_bounds()
        } else {
            self.subject.data_bounds()
        };
        let view = ViewTransformer::new(data_bbox, screen_bbox, configs.offset);
        // We have all the necessary parts for constructing the graph. With that is a job of
        // seeing what we have and what don't.
        if let Some(grid) = configs.grid {
            // If the grid has a config, use it, else, defaults to configs from the graph + defaults
            // from element.
            let grid_conf = configs.grid_configs.unwrap_or({
                GridLinesConfigBuilder::default()
                    .bbox(configs.bounding_box)
                    .color(configs.colorscheme.grid)
                    .build()
                    .expect("Default values set")
            });
            grid.draw_in_view(rl, grid_conf, &view);
        }

        // We plot the subject inside the view.
        configs.subject_configs.apply_theme(&configs.colorscheme);
        self.subject
            .draw_in_view(rl, configs.subject_configs, &view);
        // Plot the axis and the story is the same. If we have an config given by the user
        // use it, else, defaults to graph configs + element defaults.
        if let Some(axis) = configs.axis {
            let axis_conf = configs.axis_configs.unwrap_or(
                AxisConfigsBuilder::default()
                    .bbox(configs.bounding_box)
                    .color(configs.colorscheme.axis)
                    .build()
                    .expect("Default values set"),
            );
            axis.draw_in_view(rl, axis_conf, &view);
        }
    }
}
