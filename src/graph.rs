#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
use crate::{
    colorscheme::{Colorscheme, Themable},
    plottable::{
        line::{
            Axis, AxisConfigs, AxisConfigsBuilder, GridLines, GridLinesConfig,
            GridLinesConfigBuilder, TickLabels, TickLabelsBuilder, TickLabelsConfig,
        },
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
    axis: Option<Axis>,
    #[builder(setter(into, strip_option), default = "None")]
    axis_configs: Option<AxisConfigs>,
    #[builder(setter(into, strip_option), default = "None")]
    grid: Option<GridLines>,
    #[builder(setter(into, strip_option), default = "None")]
    grid_configs: Option<GridLinesConfig>,
    #[builder(default)]
    colorscheme: Colorscheme,
    #[builder(setter(into, strip_option), default = "None")]
    ticks: Option<TickLabels>,
    #[builder(setter(into, strip_option), default = "None")]
    ticks_configs: Option<TickLabelsConfig>,
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
            axis_configs: self.axis_configs.unwrap_or(None),
            grid: self.grid.unwrap_or(None),
            grid_configs: self.grid_configs.unwrap_or(None),
            colorscheme: self.colorscheme.unwrap_or_default(),
            ticks: self.ticks.unwrap_or(None),
            ticks_configs: self.ticks_configs.unwrap_or(None),
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
    fn resolve_theme(mut self) -> Self {
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

        // Ticks config: if enabled and missing config, create a themed default; otherwise theme it.
        if self.ticks.is_some() {
            match &mut self.ticks_configs {
                Some(cfg) => cfg.apply_theme(&self.colorscheme),
                None => {
                    self.ticks_configs = Some(
                        TickLabelsBuilder::default()
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

    fn plot(&self, rl: &mut raylib::prelude::RaylibDrawHandle, configs: &GraphConfig<T>) {
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
        {
            let inner_bbox = screen.inner_bbox();
            let (x, y, w, h) = scissor_rect_from_bbox(inner_bbox);
            let mut scissors = rl.begin_scissor_mode(x, y, w, h);
            // We have all the necessary parts for constructing the graph. With that is a job of
            // seeing what we have and what don't.
            if let Some(grid) = configs.grid {
                // If the grid has a config, use it, else, defaults to configs from the graph + defaults
                // from element.
                // NOTE: Should always unwrap with a configuration, if the user applied theming.
                assert!(configs.grid_configs.is_some());
                let grid_conf = configs
                    .grid_configs
                    .expect("Should always be set by the default constructor");
                grid.draw_in_view(&mut scissors, &grid_conf, &view);
            }

            if let Some(ticks) = &configs.ticks {
                assert!(configs.ticks_configs.is_some());
                let ticks_config = configs
                    .ticks_configs
                    .expect("Should always be set by the default constructor");
                ticks.draw_in_view(&mut scissors, &ticks_config, &view);
            }

            // We plot the subject inside the view.
            // configs.subject_configs.apply_theme(&configs.colorscheme);
            self.subject
                .draw_in_view(&mut scissors, &configs.subject_configs, &view);
        }
        //If we have an config given by the user
        // use it, else, defaults to graph configs + element defaults.
        // NOTE: Axis shouldn't be scissored.
        if let Some(axis) = configs.axis {
            // let axis_conf = configs.axis_configs.unwrap_or(
            //     AxisConfigsBuilder::default()
            //         .color(configs.colorscheme.axis)
            //         .build()
            //         .expect("Default values set"),
            // );
            assert!(configs.axis_configs.is_some());
            let axis_conf = configs
                .axis_configs
                .expect("Should always be set by the default constructor");
            axis.draw_in_view(rl, &axis_conf, &view);
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
