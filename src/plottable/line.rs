#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
#![forbid(unsafe_code)]
use std::{f32, ops::Range};

use derive_builder::Builder;
use raylib::prelude::*;

use crate::{
    colorscheme::Themable,
    plottable::{
        common::{get_spacing, nice_number},
        point::Datapoint,
        ticks::{Scale, TickSet, TickSpec},
        view::{DataBBox, ViewTransformer},
    },
    plotter::{ChartElement, PlotElement},
};

/// Represents a line in going `from` a point `to` another.
#[derive(Debug, Clone, Copy)]
pub struct Line {
    pub(crate) from: Datapoint,
    pub(crate) to: Datapoint,
}

impl Line {
    #[must_use]
    pub fn new(from: impl Into<Datapoint>, to: impl Into<Datapoint>) -> Self {
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
    arrow: Visibility,
    arrow_length: f32,
    arrow_width: f32,
}

impl Default for LineConfig {
    fn default() -> Self {
        let thickness = 1.5;
        Self {
            thickness,
            color: Color::WHITE,
            arrow: Visibility::Visible,
            arrow_length: 4.0 * thickness,
            arrow_width: 3.5 * thickness,
        }
    }
}

impl PlotElement for Line {
    type Config = LineConfig;
    fn plot(&self, rl: &mut RaylibDrawHandle, configs: &LineConfig) {
        match configs.arrow {
            Visibility::Visible => {
                rl.draw_line_ex(*self.from, *self.to, configs.thickness, configs.color);
                let direction = Vector2 {
                    x: self.to.x - self.from.x,
                    y: self.to.y - self.from.y,
                };
                let length = direction.length();
                if length <= 0.0 {
                    return;
                }
                let direction_norm = direction.normalized();
                let vdx = -direction_norm.y;
                let vdy = direction_norm.x;
                let p1 = Vector2::new(
                    self.to.x - configs.arrow_length * direction_norm.x + configs.arrow_width * vdx,
                    self.to.y - configs.arrow_length * direction_norm.y + configs.arrow_width * vdy,
                );
                let p2 = Vector2::new(
                    self.to.x - configs.arrow_length * direction_norm.x - configs.arrow_width * vdx,
                    self.to.y - configs.arrow_length * direction_norm.y - configs.arrow_width * vdy,
                );
                let tail = Vector2::new(self.to.x, self.to.y);
                rl.draw_triangle(p2, p1, tail, configs.color);
            }
            Visibility::Invisible => {
                rl.draw_line_ex(*self.from, *self.to, configs.thickness, configs.color);
            }
        }
    }
}

/// Definition of an Axis
#[derive(Clone, Copy, Debug)]
pub struct Axis {
    pub(crate) x_axis: Line,
    pub(crate) y_axis: Line,
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

    /// Creates a new Axis that fits the given data ranges, applying "nice number" algorithms
    /// to determine the ticks and padding.
    #[must_use]
    pub fn fitting(
        x_range: Range<f32>,
        y_range: Range<f32>,
        padding_pct: f32,
        ticks: usize,
    ) -> Self {
        let (min_x, max_x) = calculate_nice_range(
            x_range.start.min(x_range.end),
            x_range.end.max(x_range.start),
            padding_pct,
            ticks,
        );
        let (min_y, max_y) = calculate_nice_range(
            y_range.start.min(y_range.end),
            y_range.end.max(y_range.start),
            padding_pct,
            ticks,
        );

        Self {
            x_axis: Line::new(Datapoint::new(min_x, min_y), Datapoint::new(max_x, min_y)),
            y_axis: Line::new(Datapoint::new(min_x, min_y), Datapoint::new(min_x, max_y)),
        }
    }
}

/// Generates a "nice range" that fits `min` and `max`. This means that will snap, generally,
/// to numbers that are multiple of 5 or 10.
#[allow(clippy::cast_precision_loss)]
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
    let rough_step = padded_range / (ticks.max(1) as f32);
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
                Datapoint::new(value.0.start, value.1.start),
                Datapoint::new(value.0.end, value.1.start),
            ),
            y_axis: Line::new(
                Datapoint::new(value.0.start, value.1.start),
                Datapoint::new(value.0.start, value.1.end),
            ),
        }
    }
}

#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    Visible,
    Invisible,
}

/// Configs for the axis. Arrows are default to True.
#[derive(Debug, Clone, Copy, Builder)]
#[builder(pattern = "owned")]
#[builder(default)]
pub struct AxisConfigs {
    #[builder(private)]
    x_arrow: Visibility,
    #[builder(private)]
    y_arrow: Visibility,
    #[builder(private)]
    x_axis: Visibility,
    #[builder(private)]
    y_axis: Visibility,
    arrow_length: f32,
    arrow_width: f32,
    color: Color,
    thickness: f32,
}

impl AxisConfigsBuilder {
    #[must_use]
    pub fn strip_both_arrows(self) -> Self {
        Self {
            x_arrow: Some(Visibility::Invisible),
            y_arrow: Some(Visibility::Invisible),
            ..self
        }
    }

    #[must_use]
    pub fn strip_y_arrow(self) -> Self {
        Self {
            y_arrow: Some(Visibility::Invisible),
            ..self
        }
    }

    #[must_use]
    pub fn strip_x_arrow(self) -> Self {
        Self {
            x_arrow: Some(Visibility::Invisible),
            ..self
        }
    }
    /// Automatically strip arrows, because there is no axis for arrows to go into
    #[must_use]
    pub fn strip_both_axis(self) -> Self {
        Self {
            x_axis: Some(Visibility::Invisible),
            y_axis: Some(Visibility::Invisible),
            x_arrow: Some(Visibility::Invisible),
            y_arrow: Some(Visibility::Invisible),
            ..self
        }
    }

    /// Automatically strip arrows, because there is no axis for arrows to go into
    #[must_use]
    pub fn strip_y_axis(self) -> Self {
        Self {
            y_axis: Some(Visibility::Invisible),
            y_arrow: Some(Visibility::Invisible),
            ..self
        }
    }

    /// Automatically strip arrows, because there is no axis for arrows to go into
    #[must_use]
    pub fn strip_x_axis(self) -> Self {
        Self {
            x_axis: Some(Visibility::Invisible),
            x_arrow: Some(Visibility::Invisible),
            ..self
        }
    }
}

impl Default for AxisConfigs {
    fn default() -> Self {
        let thickness = 2.0;
        Self {
            x_arrow: Visibility::Visible,
            y_arrow: Visibility::Visible,
            x_axis: Visibility::Visible,
            y_axis: Visibility::Visible,
            arrow_length: 4.0 * thickness,
            color: Color::WHITE,
            thickness,
            arrow_width: 4.0 * thickness,
        }
    }
}

impl ChartElement for Axis {
    type Config = AxisConfigs;

    fn draw_in_view(
        &self,
        rl: &mut RaylibDrawHandle,
        configs: &Self::Config,
        view: &ViewTransformer,
    ) {
        let (x_line, y_line) = {
            let x_start = view.to_screen(&self.x_axis.from);
            let x_end = view.to_screen(&self.x_axis.to);
            let y_start = view.to_screen(&self.y_axis.from);
            let y_end = view.to_screen(&self.y_axis.to);
            (Line::new(*x_start, *x_end), Line::new(*y_start, *y_end))
        };

        let line_config_x = LineConfig {
            thickness: configs.thickness,
            color: configs.color,
            arrow: configs.x_arrow,
            arrow_length: configs.arrow_length,
            arrow_width: configs.arrow_width,
        };

        let line_config_y = LineConfig {
            thickness: configs.thickness,
            color: configs.color,
            arrow: configs.y_arrow,
            arrow_length: configs.arrow_length,
            arrow_width: configs.arrow_width,
        };
        match configs.x_axis {
            Visibility::Visible => {
                x_line.plot(rl, &line_config_x);
            }
            Visibility::Invisible => (),
        }
        match configs.y_axis {
            Visibility::Visible => {
                y_line.plot(rl, &line_config_y);
            }
            Visibility::Invisible => (),
        }
    }

    fn data_bounds(&self) -> DataBBox {
        DataBBox::from_min_max(
            (
                self.x_axis.from.x.min(self.x_axis.to.x),
                self.y_axis.from.y.min(self.y_axis.to.y),
            ),
            (
                self.x_axis.from.x.max(self.x_axis.to.x),
                self.y_axis.from.y.max(self.y_axis.to.y),
            ),
        )
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
    thickness: f32,
    max_ticks: usize,
}

impl Default for GridLinesConfig {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            alpha: 0.3,
            thickness: 1.0,
            max_ticks: 10,
        }
    }
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
        let start = view.to_screen(&Datapoint::new(data_x, data_y_start));
        let end = view.to_screen(&Datapoint::new(data_x, data_y_end));

        let color = config.color.alpha(config.alpha);
        rl.draw_line_ex(*start, *end, config.thickness, color);
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

        let start = view.to_screen(&Datapoint::new(data_x_start, data_y));
        let end = view.to_screen(&Datapoint::new(data_x_end, data_y));

        let color = config.color.alpha(config.alpha);
        rl.draw_line_ex(*start, *end, config.thickness, color);
    }

    fn plot_vertical(
        &self,
        rl: &mut RaylibDrawHandle,
        config: &GridLinesConfig,
        sep: Separation,
        view: &ViewTransformer,
    ) {
        let spacing = get_spacing(self.axis.length_x_axis(), sep, config.max_ticks);
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
        sep: Separation,
        view: &ViewTransformer,
    ) {
        let spacing = get_spacing(self.axis.length_y_axis(), sep, config.max_ticks);
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
#[allow(clippy::cast_precision_loss)]
impl ChartElement for GridLines {
    type Config = GridLinesConfig;

    fn draw_in_view(
        &self,
        rl: &mut RaylibDrawHandle,
        configs: &GridLinesConfig,
        view: &ViewTransformer,
    ) {
        match &self.orientation {
            Orientation::Vertical { separation } => {
                self.plot_vertical(rl, configs, *separation, view);
            }
            Orientation::Horizontal { separation } => {
                self.plot_horizontal(rl, configs, *separation, view);
            }
            Orientation::Both {
                separation_x,
                separation_y,
            } => {
                self.plot_vertical(rl, configs, *separation_x, view);
                self.plot_horizontal(rl, configs, *separation_y, view);
            }
        }
    }

    fn data_bounds(&self) -> DataBBox {
        self.axis.data_bounds()
    }
}

impl Themable for GridLinesConfig {
    fn apply_theme(&mut self, scheme: &crate::colorscheme::Colorscheme) {
        self.color = scheme.grid;
    }
}

#[derive(Clone, Debug, Copy)]
pub struct TickLabels {
    pub(crate) axis: Axis,
}

impl TickLabels {
    pub fn new(axis: Axis) -> Self {
        Self { axis }
    }
}

#[derive(Debug, Clone, Copy, Builder)]
#[builder(pattern = "owned")]
#[builder(default, name = "TickLabelsBuilder")]
pub struct TickLabelsConfig {
    color: Color,
    alpha: f32,
    major_size: f32,
    minor_size: f32,
    max_ticks: usize,
    separation: Separation,
    #[builder(private)]
    x_axis: Visibility,
    #[builder(default = "Scale::Linear", private)]
    x_axis_scale: Scale, // Only matter if x axis is visible, else is ignored
    #[builder(private)]
    y_axis: Visibility,
    #[builder(default = "Scale::Linear", private)]
    y_axis_scale: Scale, // Only matter if y axis is visible, else is ignored
}

impl TickLabelsBuilder {
    #[must_use]
    pub fn with_x_scale(self, scale: Scale) -> Self {
        Self {
            x_axis: Some(Visibility::Visible),
            x_axis_scale: Some(scale),
            ..self
        }
    }

    #[must_use]
    pub fn with_y_scale(self, scale: Scale) -> Self {
        Self {
            y_axis: Some(Visibility::Visible),
            y_axis_scale: Some(scale),
            ..self
        }
    }

    #[must_use]
    pub fn with_both_axis_scale(self, scale: Scale) -> Self {
        Self {
            y_axis: Some(Visibility::Visible),
            y_axis_scale: Some(scale),
            x_axis: Some(Visibility::Visible),
            x_axis_scale: Some(scale),
            ..self
        }
    }
}

impl Default for TickLabelsConfig {
    fn default() -> Self {
        Self {
            color: Color::WHITE,
            alpha: 0.3,
            major_size: 5.0,
            minor_size: 3.0,
            max_ticks: 10,
            separation: Separation::Auto,
            x_axis: Visibility::Invisible,
            y_axis: Visibility::Invisible,
            x_axis_scale: Scale::Linear, // Do not matter as x axis is invisible
            y_axis_scale: Scale::Linear,
        }
    }
}

impl ChartElement for TickLabels {
    type Config = TickLabelsConfig;

    fn draw_in_view(
        &self,
        rl: &mut RaylibDrawHandle,
        configs: &Self::Config,
        view: &ViewTransformer,
    ) {
        let data_bounds = self.data_bounds();
        match configs.x_axis {
            Visibility::Visible => {
                let tickset = TickSet::generate_ticks(
                    data_bounds.minimum.x,
                    data_bounds.maximum.x,
                    TickSpec {
                        scale: configs.x_axis_scale,
                        max_ticks: configs.max_ticks,
                        separation: configs.separation,
                    },
                );
                for ticks in tickset.ticks {
                    let screen_point = view.to_screen(&(ticks.value, data_bounds.minimum.y).into());
                    rl.draw_line_v(
                        Vector2::new(
                            screen_point.x,
                            screen_point.y
                                - if ticks.major {
                                    configs.major_size
                                } else {
                                    configs.minor_size
                                },
                        ),
                        Vector2::new(
                            screen_point.x,
                            screen_point.y
                                + if ticks.major {
                                    configs.major_size
                                } else {
                                    configs.minor_size
                                },
                        ),
                        configs.color,
                    );
                }
            }
            Visibility::Invisible => {}
        }

        match configs.y_axis {
            Visibility::Visible => {
                let tickset = TickSet::generate_ticks(
                    data_bounds.minimum.y,
                    data_bounds.maximum.y,
                    TickSpec {
                        scale: configs.y_axis_scale,
                        max_ticks: configs.max_ticks,
                        separation: configs.separation,
                    },
                );
                for ticks in tickset.ticks {
                    let screen_point = view.to_screen(&(ticks.value, data_bounds.minimum.x).into());
                    rl.draw_line_v(
                        Vector2::new(
                            screen_point.x
                                - if ticks.major {
                                    configs.major_size
                                } else {
                                    configs.minor_size
                                },
                            screen_point.y,
                        ),
                        Vector2::new(
                            screen_point.x
                                + if ticks.major {
                                    configs.major_size
                                } else {
                                    configs.minor_size
                                },
                            screen_point.y,
                        ),
                        configs.color,
                    );
                }
            }
            Visibility::Invisible => {}
        }
    }

    fn data_bounds(&self) -> DataBBox {
        self.axis.data_bounds()
    }
}

impl Themable for TickLabelsConfig {
    fn apply_theme(&mut self, scheme: &crate::colorscheme::Colorscheme) {
        self.color = scheme.axis;
    }
}
