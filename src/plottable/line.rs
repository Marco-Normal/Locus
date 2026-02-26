//! Lines, axes, grid lines, tick labels, and their configurations.
//!
//! This module contains the foundational geometric elements that make up the
//! chrome of a graph:
//!
//! * [`Line`] : a directed segment between two points, optionally with an
//!   arrowhead.
//! * [`Axis`] : a pair of perpendicular lines representing the x and y axes,
//!   with automatic "nice number" range fitting.
//! * [`GridLines`] : evenly spaced reference lines aligned to the axis, drawn
//!   behind the data.
//! * [`TickLabels`] : small marks along each axis with formatted numeric
//!   labels.
//!
//! Each element has an associated `*Config` / `*Configs` type (built via
//! `derive_builder`) and implements either [`PlotElement`] or
//! [`ChartElement`] depending on whether it needs a view transform.

#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
#![forbid(unsafe_code)]
use std::{f32, ops::Range};

use derive_builder::Builder;
use raylib::prelude::*;

use crate::{
    TextLabel,
    colorscheme::Themable,
    plottable::{
        common::{get_spacing, nice_number},
        point::{Datapoint, Screenpoint},
        text::{Anchor, TextStyle},
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
/// Configurations for a [`Line`].
///
/// Controls thickness, color, and whether an arrowhead is rendered at the
/// destination end. When `color` is `None` it is resolved from the active
/// [`Colorscheme`](crate::colorscheme::Colorscheme) at theme-application
/// time.
///
/// Built via [`LineConfigBuilder`]:
///
/// ```rust,ignore
/// let cfg = LineConfigBuilder::default()
///     .thickness(2.0)
///     .color(Color::RED)
///     .arrow(Visibility::Invisible)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone, Copy, Builder)]
#[builder(pattern = "owned")]
#[builder(default)]
pub struct LineConfig {
    /// Line width in pixels.
    pub thickness: f32,
    /// Explicit color. `None` means "use the theme default".
    #[builder(setter(into, strip_option))]
    pub color: Option<Color>,
    /// Whether to draw an arrowhead at the `to` end.
    pub arrow: Visibility,
    /// Length of the arrowhead along the line direction (pixels).
    pub arrow_length: f32,
    /// Half-width of the arrowhead perpendicular to the line (pixels).
    pub arrow_width: f32,
}

impl Default for LineConfig {
    fn default() -> Self {
        let thickness = 1.5;
        Self {
            thickness,
            color: None,
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
                rl.draw_line_ex(
                    *self.from,
                    *self.to,
                    configs.thickness,
                    configs.color.unwrap_or(Color::BLACK),
                );
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
                rl.draw_triangle(p2, p1, tail, configs.color.unwrap_or(Color::BLACK));
            }
            Visibility::Invisible => {
                rl.draw_line_ex(
                    *self.from,
                    *self.to,
                    configs.thickness,
                    configs.color.unwrap_or(Color::BLACK),
                );
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

/// Toggle for visual elements that can be shown or hidden.
#[derive(Debug, Clone, Copy)]
pub enum Visibility {
    /// The element is drawn.
    Visible,
    /// The element is suppressed.
    Invisible,
}

/// Configuration for the pair of axis lines.
///
/// Individual axes and their arrowheads can be toggled via the builder
/// helpers [`strip_x_axis`](AxisConfigsBuilder::strip_x_axis),
/// [`strip_y_axis`](AxisConfigsBuilder::strip_y_axis), and
/// [`strip_both_arrows`](AxisConfigsBuilder::strip_both_arrows).
///
/// When `color` is `None`, it is resolved from
/// [`Colorscheme::axis`](crate::colorscheme::Colorscheme::axis) during
/// theme application.
#[derive(Debug, Clone, Copy, Builder)]
#[builder(pattern = "owned")]
#[builder(default)]
pub struct AxisConfigs {
    /// Visibility of the x-axis arrowhead.
    #[builder(private)]
    pub x_arrow: Visibility,
    /// Visibility of the y-axis arrowhead.
    #[builder(private)]
    pub y_arrow: Visibility,
    /// Visibility of the x-axis line itself.
    #[builder(private)]
    pub x_axis: Visibility,
    /// Visibility of the y-axis line itself.
    #[builder(private)]
    pub y_axis: Visibility,
    /// Length of arrowheads in pixels.
    pub arrow_length: f32,
    /// Width of arrowheads in pixels.
    pub arrow_width: f32,
    /// Explicit color. `None` means "use theme axis color".
    #[builder(setter(into, strip_option))]
    pub color: Option<Color>,
    /// Line thickness in pixels.
    pub thickness: f32,
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
            color: None,
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
        match &self.color {
            Some(_) => {}
            None => {
                self.color = Some(scheme.axis);
            }
        }
    }
}

/// Controls which directions grid lines are drawn and with what spacing.
#[derive(Debug, Clone, Copy)]
pub enum Orientation {
    /// Only vertical grid lines (perpendicular to the x-axis).
    Vertical {
        /// Spacing strategy between consecutive vertical lines.
        separation: Separation,
    },
    /// Only horizontal grid lines (perpendicular to the y-axis).
    Horizontal {
        /// Spacing strategy between consecutive horizontal lines.
        separation: Separation,
    },
    /// Both vertical and horizontal grid lines (the default).
    Both {
        /// Spacing for vertical lines.
        separation_x: Separation,
        /// Spacing for horizontal lines.
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

/// Strategy for spacing grid lines or tick marks along an axis.
#[derive(Debug, Clone, Copy, Default)]
pub enum Separation {
    /// Let the library choose a "nice" spacing automatically.
    #[default]
    Auto,
    /// Use an explicit spacing value in data units.
    Value(f32),
}

/// Grid lines drawn behind the data to aid visual reading.
///
/// Constructed from an [`Axis`] (which defines the data range) and an
/// [`Orientation`] (which controls direction and spacing). Implements
/// [`ChartElement`] and is rendered via a [`ViewTransformer`].
#[derive(Debug, Clone, Copy)]
pub struct GridLines {
    pub(crate) axis: Axis,
    pub(crate) orientation: Orientation,
}

impl GridLines {
    /// Create grid lines for `axis` in the given `orientation`.
    #[must_use]
    pub fn new(axis: Axis, orientation: Orientation) -> Self {
        Self { axis, orientation }
    }
}

/// Configuration for [`GridLines`] rendering.
///
/// When `color` is `None` it is resolved from
/// [`Colorscheme::grid`](crate::colorscheme::Colorscheme::grid) during
/// theme application.
#[derive(Debug, Clone, Copy, Builder)]
#[builder(pattern = "owned")]
#[builder(default)]
pub struct GridLinesConfig {
    /// Explicit grid color. `None` means "use theme grid color".
    #[builder(setter(strip_option, into))]
    pub color: Option<Color>,
    /// Alpha multiplier applied on top of the color's own alpha.
    pub alpha: f32,
    /// Line thickness in pixels.
    pub thickness: f32,
    /// Maximum number of grid lines per axis (used by the auto-spacing
    /// algorithm).
    pub max_ticks: usize,
}

impl Default for GridLinesConfig {
    fn default() -> Self {
        Self {
            color: None,
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

        let color = config.color.unwrap_or(Color::BLACK).alpha(config.alpha);
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

        let color = config.color.unwrap_or(Color::BLACK).alpha(config.alpha);
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
        match &self.color {
            Some(_) => {}
            None => self.color = Some(scheme.grid),
        }
    }
}

/// Small marks along each axis with formatted numeric labels.
///
/// `TickLabels` combines tick mark rendering with optional text labels
/// placed next to each major tick. It supports linear, logarithmic, and
/// symmetric-log scales via the [`Scale`] enum in [`ticks`](super::ticks).
///
/// Constructed from an [`Axis`] and configured through
/// [`TickLabelsConfig`] / [`TickLabelsBuilder`].
#[derive(Clone, Debug, Copy)]
pub struct TickLabels {
    pub(crate) axis: Axis,
}

impl TickLabels {
    /// Create tick labels for the given `axis`.
    #[must_use]
    pub fn new(axis: Axis) -> Self {
        Self { axis }
    }
}

/// Configuration for [`TickLabels`] rendering.
///
/// Controls which axes display ticks, the scale type (linear, log,
/// symmetric-log), mark sizes, label text style, and spacing.
///
/// When `color` is `None` it is resolved from
/// [`Colorscheme::axis`](crate::colorscheme::Colorscheme::axis). The label
/// text style is separately themed from
/// [`Colorscheme::text`](crate::colorscheme::Colorscheme::text).
#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned")]
#[builder(default, name = "TickLabelsBuilder")]
pub struct TickLabelsConfig {
    /// Explicit tick mark color. `None` means "use theme axis color".
    #[builder(setter(strip_option, into))]
    pub color: Option<Color>,
    /// Alpha multiplier for tick marks.
    pub alpha: f32,
    /// Length of major tick marks in pixels.
    pub major_size: f32,
    /// Length of minor tick marks in pixels (log/symlog scales).
    pub minor_size: f32,
    /// Maximum number of ticks per axis.
    pub max_ticks: usize,
    /// Spacing strategy for tick placement.
    pub separation: Separation,
    /// Visibility of x-axis ticks.
    #[builder(private)]
    pub x_axis: Visibility,
    /// Scale type for x-axis ticks (linear, log, or symlog).
    #[builder(default = "Scale::Linear", private)]
    pub x_axis_scale: Scale,
    /// Visibility of y-axis ticks.
    #[builder(private)]
    pub y_axis: Visibility,
    /// Scale type for y-axis ticks (linear, log, or symlog).
    #[builder(default = "Scale::Linear", private)]
    pub y_axis_scale: Scale,

    /// Whether to draw numeric labels next to tick marks.
    pub show_labels: bool,
    /// Text style applied to tick labels. Themed via [`Colorscheme::text`](crate::colorscheme::Colorscheme::text).
    pub label_style: TextStyle,
    /// Gap in pixels between the tick mark and the start of the label text.
    pub label_offset: f32,
    /// Rotation in degrees for x-axis tick labels (useful for long labels).
    pub label_rotation: f32,
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
    #[must_use]
    pub fn strip_x_axis(self) -> Self {
        Self {
            x_axis: Some(Visibility::Invisible),
            ..self
        }
    }

    #[must_use]
    pub fn strip_y_axis(self) -> Self {
        Self {
            y_axis: Some(Visibility::Invisible),
            ..self
        }
    }
}

impl Default for TickLabelsConfig {
    fn default() -> Self {
        Self {
            color: None,
            alpha: 1.0,
            major_size: 7.0,
            minor_size: 5.0,
            max_ticks: 10,
            separation: Separation::Auto,
            x_axis: Visibility::Visible,
            y_axis: Visibility::Visible,
            x_axis_scale: Scale::Linear,
            y_axis_scale: Scale::Linear,
            show_labels: true,
            label_style: TextStyle {
                font_size: 14.0,
                alpha: 1.0,
                color: None,
                spacing: 1.0,
                font: None,
                anchor: Anchor::TOP_CENTER,
                rotation: 0.0,
                offset: Vector2::new(0.0, 0.0),
            },
            label_offset: 4.0,
            label_rotation: 0.0,
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
                for tick in &tickset.ticks {
                    if !(data_bounds.minimum.x..data_bounds.maximum.x).contains(&tick.value) {
                        continue;
                    }
                    let screen_point = view.to_screen(&(tick.value, data_bounds.minimum.y).into());
                    let mark_len = if tick.major {
                        configs.major_size
                    } else {
                        configs.minor_size
                    };
                    rl.draw_line_v(
                        Vector2::new(screen_point.x, screen_point.y),
                        Vector2::new(screen_point.x, screen_point.y + mark_len),
                        configs.color.unwrap_or(Color::BLACK),
                    );

                    // Draw tick label text (major ticks only, unless label is non-empty)
                    if configs.show_labels && tick.major && !tick.label.is_empty() {
                        let mut style = configs.label_style.clone();
                        style.anchor = Anchor::TOP_CENTER;
                        style.rotation = configs.label_rotation;
                        let origin = Screenpoint::new(
                            screen_point.x,
                            screen_point.y + mark_len + configs.label_offset,
                        );
                        let text = TextLabel::new(&tick.label, origin);
                        text.plot(rl, &style);
                    }
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
                for tick in &tickset.ticks {
                    if !(data_bounds.minimum.y..data_bounds.maximum.y).contains(&tick.value) {
                        continue;
                    }
                    let screen_point = view.to_screen(&(data_bounds.minimum.x, tick.value).into());
                    let mark_len = if tick.major {
                        configs.major_size
                    } else {
                        configs.minor_size
                    };
                    rl.draw_line_v(
                        Vector2::new(screen_point.x - mark_len, screen_point.y),
                        Vector2::new(screen_point.x, screen_point.y),
                        configs.color.unwrap_or(Color::BLACK),
                    );

                    // Draw tick label text
                    if configs.show_labels && tick.major && !tick.label.is_empty() {
                        let mut style = configs.label_style.clone();
                        style.anchor = Anchor::RIGHT_MIDDLE;
                        let origin = Screenpoint::new(
                            screen_point.x - mark_len - configs.label_offset,
                            screen_point.y,
                        );
                        let text = TextLabel::new(&tick.label, origin);
                        text.plot(rl, &style);
                    }
                }
            }
            Visibility::Invisible => {}
        }
    }

    fn data_bounds(&self) -> DataBBox {
        self.axis.data_bounds()
    }
}

/// Follows the color of the axis for tick marks; themes label text via `colorscheme.text`.
impl Themable for TickLabelsConfig {
    fn apply_theme(&mut self, scheme: &crate::colorscheme::Colorscheme) {
        if self.color.is_none() {
            self.color = Some(scheme.axis);
        }
        self.label_style.apply_theme(scheme);
    }
}
