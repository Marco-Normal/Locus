#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
#![forbid(unsafe_code)]
use std::{f32, ops::Range};

use derive_builder::Builder;
use raylib::prelude::*;

use crate::{
    colorscheme::Themable,
    dataset::Dataset,
    plottable::{
        point::Point,
        view::{BBox, Offsets, ViewTransformer},
    },
    plotter::{ChartElement, PlotElement},
};

/// Represents a line in going `from` a point `to` another.
#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub(crate) from: Point,
    pub(crate) to: Point,
}

impl Line {
    #[must_use]
    pub fn new(from: impl Into<Point>, to: impl Into<Point>) -> Self {
        Self {
            from: from.into(),
            to: to.into(),
        }
    }
}
/// Configurations for a line. The line could have an arrow in it's tip, and
/// arguments from such are also inside line config. As default, it has no
/// arrow.
#[derive(Debug, Clone, Copy, Builder)]
#[builder(pattern = "owned")]
#[builder(default)]
pub struct LineConfig {
    thickness: f32,
    color: Color,
    arrow: bool,
    arrow_length: f32,
    arrow_width: f32,
    offsets: Offsets,
}

impl Default for LineConfig {
    fn default() -> Self {
        let thickness = 1.5;
        Self {
            thickness,
            color: Color::WHITE,
            arrow: true,
            arrow_length: 4.0 * thickness,
            arrow_width: 3.5 * thickness,
            offsets: Offsets::default(),
        }
    }
}

// impl PlotConfig for LineConfig {
//     fn with_bounds(&mut self, by: BBox) {
//         self.bbox = by;
//     }

//     fn with_offset(&mut self, by: Offsets) {
//         self.offsets = by;
//     }
// }

impl PlotElement for Line {
    type Config = LineConfig;
    fn plot(&self, rl: &mut RaylibDrawHandle, configs: Self::Config) {
        let from = Vector2::new(
            self.from.x + configs.offsets.offset_x,
            self.from.y + configs.offsets.offset_y,
        );
        let to = Vector2::new(
            self.to.x + configs.offsets.offset_x,
            self.to.y + configs.offsets.offset_y,
        );
        if configs.arrow {
            rl.draw_line_ex(from, to, configs.thickness, configs.color);
            let direction = Vector2 {
                x: to.x - from.x,
                y: to.y - from.y,
            };
            let length = direction.length();
            if length <= 0.0 {
                return;
            }
            let direction_norm = direction.normalized();
            let vdx = -direction_norm.y;
            let vdy = direction_norm.x;
            let p1 = Vector2::new(
                to.x - configs.arrow_length * direction_norm.x + configs.arrow_width * vdx,
                to.y - configs.arrow_length * direction_norm.y + configs.arrow_width * vdy,
            );
            let p2 = Vector2::new(
                to.x - configs.arrow_length * direction_norm.x - configs.arrow_width * vdx,
                to.y - configs.arrow_length * direction_norm.y - configs.arrow_width * vdy,
            );
            let tail = Vector2::new(to.x, to.y);
            rl.draw_triangle(p2, p1, tail, configs.color);
        } else {
            rl.draw_line_ex(from, to, configs.thickness, configs.color);
        }
    }
}

/// Definition of an Axis
#[derive(Clone, Copy, Debug)]
pub struct Axis {
    pub(crate) x_axis: Line,
    pub(crate) y_axis: Line,
}

/// Given a dataset, generate axis from it, snapping to "nice numbers", instead of
/// exclusively going from max to min.
impl From<Dataset> for Axis {
    fn from(value: Dataset) -> Self {
        Self {
            x_axis: Line {
                from: Point {
                    x: value.range_min.x,
                    y: value.range_min.y,
                },
                to: Point {
                    x: value.range_max.x,
                    y: value.range_min.y,
                },
            },
            y_axis: Line {
                from: Point {
                    x: value.range_min.x,
                    y: value.range_min.y,
                },
                to: Point {
                    x: value.range_min.x,
                    y: value.range_max.y,
                },
            },
        }
    }
}

impl From<&Dataset> for Axis {
    fn from(value: &Dataset) -> Self {
        Self::nice_fit(value, 0.01, 15)
    }
}

impl Axis {
    #[must_use]
    pub fn new(x_axis: Line, y_axis: Line) -> Self {
        Self { x_axis, y_axis }
    }
    fn length_x_axis(&self) -> f32 {
        (self.x_axis.to.x - self.x_axis.from.x).abs()
    }

    fn length_y_axis(&self) -> f32 {
        (self.y_axis.to.y - self.y_axis.from.y).abs()
    }

    fn nice_fit(dataset: &Dataset, padding_pct: f32, target_ticks: usize) -> Self {
        let (nice_min_x, nice_max_x) = calculate_nice_range(
            dataset.range_min.x,
            dataset.range_max.x,
            padding_pct,
            target_ticks,
        );
        let (nice_min_y, nice_max_y) = calculate_nice_range(
            dataset.range_min.y,
            dataset.range_max.y,
            padding_pct,
            target_ticks,
        );
        Axis {
            x_axis: Line {
                from: Point {
                    x: nice_min_x,
                    y: nice_min_y,
                },
                to: Point {
                    x: nice_max_x,
                    y: nice_min_y,
                },
            },
            y_axis: Line {
                from: Point {
                    x: nice_min_x,
                    y: nice_min_y,
                },
                to: Point {
                    x: nice_min_x,
                    y: nice_max_y,
                },
            },
        }
    }
}

/// Generates a "nice range" that fits `min` and `max`. This means that will snap, generally,
/// to numbers that are multiple of 5 or 10.
fn calculate_nice_range(min: f32, max: f32, padding_pct: f32, ticks: usize) -> (f32, f32) {
    if (min - max).abs() < f32::EPSILON {
        return (min - 1.0, max + 1.0); // Handle single-point datasets
    }

    let range = max - min;

    // Add padding
    let padding = range * padding_pct;
    let padded_min = min - padding;
    let padded_max = max + padding;

    // Calculate a "nice" step size based on the PADDED range
    let padded_range = padded_max - padded_min;
    let rough_step = padded_range / (ticks as f32);
    let step = nice_number(rough_step, true);

    // Snap to the nearest grid line outside the data
    let nice_min = (padded_min / step).floor() * step;
    let nice_max = (padded_max / step).ceil() * step;

    (nice_min, nice_max)
}
impl From<(Range<f32>, Range<f32>)> for Axis {
    fn from(value: (Range<f32>, Range<f32>)) -> Self {
        Axis {
            x_axis: Line::new(
                Point::new(value.0.start, value.1.start),
                Point::new(value.0.end, value.1.start),
            ),
            y_axis: Line::new(
                Point::new(value.0.start, value.1.start),
                Point::new(value.0.start, value.1.end),
            ),
        }
    }
}
/// Configs for the axis. Arrows are default to True.
#[derive(Debug, Clone, Copy, Builder)]
#[builder(pattern = "owned")]
#[builder(default)]
pub struct AxisConfigs {
    arrows: bool,
    arrow_length: f32,
    arrow_width: f32,
    color: Color,
    thickness: f32,
    offsets: Offsets,
    bbox: BBox,
}

impl Default for AxisConfigs {
    fn default() -> Self {
        let thickness = 2.0;
        Self {
            arrows: true,
            arrow_length: 4.0 * thickness,
            color: Color::WHITE,
            thickness,
            arrow_width: 4.0 * thickness,
            offsets: Offsets::default(),
            bbox: BBox::default(),
        }
    }
}

impl From<AxisConfigs> for LineConfig {
    fn from(value: AxisConfigs) -> Self {
        Self {
            thickness: value.thickness,
            color: value.color,
            arrow: value.arrows,
            arrow_length: value.arrow_length,
            arrow_width: value.arrow_width,
            offsets: value.offsets,
        }
    }
}

impl ChartElement for Axis {
    type Config = AxisConfigs;

    fn draw_in_view(
        &self,
        rl: &mut RaylibDrawHandle,
        configs: Self::Config,
        view: &ViewTransformer,
    ) {
        let (x_line, y_line) = {
            let x_start = view.to_screen(&self.x_axis.from);
            let x_end = view.to_screen(&self.x_axis.to);
            let y_start = view.to_screen(&self.y_axis.from);
            let y_end = view.to_screen(&self.y_axis.to);
            (Line::new(x_start, x_end), Line::new(y_start, y_end))
        };

        let line_config = LineConfig {
            thickness: configs.thickness,
            color: configs.color,
            arrow: configs.arrows,
            arrow_length: configs.arrow_length,
            arrow_width: configs.arrow_width,
            offsets: configs.offsets,
        };

        x_line.plot(rl, line_config);
        y_line.plot(rl, line_config);
    }

    fn data_bounds(&self) -> BBox {
        BBox {
            maximum: Point {
                x: self.x_axis.from.x.max(self.x_axis.to.x),
                y: self.y_axis.from.y.max(self.y_axis.to.y),
            },
            minimum: Point {
                x: self.x_axis.from.x.min(self.x_axis.to.x),
                y: self.y_axis.from.y.min(self.y_axis.to.y),
            },
        }
    }
}

impl Themable for AxisConfigs {
    fn apply_theme(&mut self, scheme: &crate::colorscheme::Colorscheme) {
        self.color = scheme.axis;
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    Vertical {
        separation: Separation,
    },
    Horizontal {
        separation: Separation,
    },
    Both {
        separation_x: Separation,
        separation_y: Separation,
    },
}
impl Default for Orientation {
    fn default() -> Self {
        Self::Both {
            separation_x: Separation::Auto,
            separation_y: Separation::Auto,
        }
    }
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Separation {
    #[default]
    Auto,
    Value(f32),
}

#[derive(Debug, Clone, Copy)]
pub struct GridLines {
    pub(crate) axis: Axis,
    pub(crate) orientation: Orientation,
}

impl GridLines {
    #[must_use]
    pub fn new(axis: Axis, orientation: Orientation) -> Self {
        Self { axis, orientation }
    }
}

#[derive(Debug, Clone, Copy, Builder)]
#[builder(pattern = "owned")]
#[builder(default)]
pub struct GridLinesConfig {
    color: Color,
    alpha: f32,
    grid_offset: Offsets,
    plot_offset: Offsets,
    thickness: f32,
    max_ticks: usize,
    bbox: BBox,
}

impl Default for GridLinesConfig {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            alpha: 0.3,
            grid_offset: Offsets::default(),
            plot_offset: Offsets::default(),
            thickness: 1.0,
            max_ticks: 10,
            bbox: BBox::default(),
        }
    }
}

fn nice_number(value: f32, round: bool) -> f32 {
    let exponent = value.log10().floor();
    let fraction = value / 10.0_f32.powf(exponent);

    let nice_fraction = if round {
        if fraction < 1.5 {
            1.0
        } else if fraction < 3.0 {
            2.0
        } else if fraction < 7.0 {
            5.0
        } else {
            10.0
        }
    } else if fraction <= 1.0 {
        1.0
    } else if fraction <= 2.0 {
        2.0
    } else if fraction <= 5.0 {
        5.0
    } else {
        10.0
    };

    nice_fraction * 10.0_f32.powf(exponent)
}

impl GridLines {
    /// Internal helper to draw a single vertical line
    fn draw_v_line(
        &self,
        rl: &mut RaylibDrawHandle,
        data_x: f32,
        config: &GridLinesConfig,
        view: &ViewTransformer,
    ) {
        // The line goes from bottom of Y-axis to top of Y-axis (in Data units)
        let data_y_start = self.axis.y_axis.from.y;
        let data_y_end = self.axis.y_axis.to.y;

        // Transform both ends to Screen Space
        let start = view.to_screen(&Point::new(data_x, data_y_start));
        let end = view.to_screen(&Point::new(data_x, data_y_end));

        // Apply offsets if needed (usually view handles this, but plot_offset is for nudging)
        let final_start = Vector2::new(
            start.x + config.plot_offset.offset_x,
            start.y + config.plot_offset.offset_y,
        );
        let final_end = Vector2::new(
            end.x + config.plot_offset.offset_x,
            end.y + config.plot_offset.offset_y,
        );

        let color = config.color.alpha(config.alpha);
        rl.draw_line_ex(final_start, final_end, config.thickness, color);
    }

    fn draw_h_line(
        &self,
        rl: &mut RaylibDrawHandle,
        data_y: f32,
        config: &GridLinesConfig,
        view: &ViewTransformer,
    ) {
        let data_x_start = self.axis.x_axis.from.x;
        let data_x_end = self.axis.x_axis.to.x;

        let start = view.to_screen(&Point::new(data_x_start, data_y));
        let end = view.to_screen(&Point::new(data_x_end, data_y));

        let final_start = Vector2::new(
            start.x + config.plot_offset.offset_x,
            start.y + config.plot_offset.offset_y,
        );
        let final_end = Vector2::new(
            end.x + config.plot_offset.offset_x,
            end.y + config.plot_offset.offset_y,
        );

        let color = config.color.alpha(config.alpha);
        rl.draw_line_ex(final_start, final_end, config.thickness, color);
    }

    fn get_spacing(&self, length: f32, separation: Separation, max_ticks: usize) -> f32 {
        match separation {
            Separation::Value(v) => v,
            Separation::Auto => {
                let rough_spacing = length / (max_ticks as f32).max(1.0);
                nice_number(rough_spacing, true)
            }
        }
    }

    fn plot_vertical(
        &self,
        rl: &mut RaylibDrawHandle,
        config: &GridLinesConfig,
        sep: &Separation,
        view: &ViewTransformer,
    ) {
        let spacing = self.get_spacing(self.axis.length_x_axis(), *sep, config.max_ticks);
        let (max, min) = (
            self.axis.x_axis.from.x.max(self.axis.x_axis.to.x),
            self.axis.x_axis.from.x.min(self.axis.x_axis.to.x),
        );

        // Find the first "nice" multiple of spacing after or at start
        let mut pos = (min / spacing).ceil() * spacing;

        while pos <= max {
            self.draw_v_line(rl, pos, config, view);
            pos += spacing;
        }
    }

    fn plot_horizontal(
        &self,
        rl: &mut RaylibDrawHandle,
        config: &GridLinesConfig,
        sep: &Separation,
        view: &ViewTransformer,
    ) {
        let spacing = self.get_spacing(self.axis.length_y_axis(), *sep, config.max_ticks);
        let (max, min) = (
            self.axis.y_axis.from.y.max(self.axis.y_axis.to.y),
            self.axis.y_axis.from.y.min(self.axis.y_axis.to.y),
        );

        // Note: Check if your Y-axis grows up or down.
        // This assumes 'from' is the smaller value.
        let mut pos = (min / spacing).ceil() * spacing;
        while pos <= max {
            self.draw_h_line(rl, pos, config, view);
            pos += spacing;
        }
    }
}

impl ChartElement for GridLines {
    type Config = GridLinesConfig;

    fn draw_in_view(
        &self,
        rl: &mut RaylibDrawHandle,
        configs: Self::Config,
        view: &ViewTransformer,
    ) {
        match &self.orientation {
            Orientation::Vertical { separation } => {
                self.plot_vertical(rl, &configs, separation, view);
            }
            Orientation::Horizontal { separation } => {
                self.plot_horizontal(rl, &configs, separation, view);
            }
            Orientation::Both {
                separation_x,
                separation_y,
            } => {
                self.plot_vertical(rl, &configs, separation_x, view);
                self.plot_horizontal(rl, &configs, separation_y, view);
            }
        }
    }

    fn data_bounds(&self) -> BBox {
        self.axis.data_bounds()
    }
}

impl Themable for GridLinesConfig {
    fn apply_theme(&mut self, scheme: &crate::colorscheme::Colorscheme) {
        self.color = scheme.grid;
    }
}
