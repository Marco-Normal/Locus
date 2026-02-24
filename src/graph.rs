#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
use crate::{
    colorscheme::{Colorscheme, Themable},
    plottable::{
        annotation::Annotation,
        legend::{Legend, LegendEntry},
        line::{Axis, GridLines, TickLabels},
        text::{Anchor, TextStyle, TextStyleBuilder},
        view::{ScreenBBox, ViewTransformer, Viewport},
    },
    plotter::{ChartElement, PlotElement},
};
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
#[derive(Debug, Clone)]
pub struct GraphConfig<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default + Themable,
{
    subject_configs: T::Config,
    viewport: Viewport,
    axis: Option<ConfiguredElement<Axis>>,
    grid: Option<ConfiguredElement<GridLines>>,
    colorscheme: Colorscheme,
    ticks: Option<ConfiguredElement<TickLabels>>,
    title: Option<(String, TextStyle)>,
    xlabel: Option<(String, TextStyle)>,
    ylabel: Option<(String, TextStyle)>,
    legend: Option<Legend>,
    annotations: Vec<Annotation>,
}

// ── Error type for GraphBuilder ──────────────────────────────────────

/// Error returned when `GraphBuilder::build()` fails.
#[derive(Debug, Clone)]
pub struct GraphBuilderError(String);

impl std::fmt::Display for GraphBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GraphBuilderError: {}", self.0)
    }
}

impl std::error::Error for GraphBuilderError {}

// ── GraphBuilder ─────────────────────────────────────────────────────

/// Ergonomic builder for `GraphConfig`.
///
/// ```ignore
/// GraphBuilder::default()
///     .viewport(vp)
///     .axis(my_axis)
///     .title("My Chart")
///     .xlabel("X")
///     .ylabel("Y")
///     .build()
///     .unwrap();
/// ```
pub struct GraphBuilder<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default + Themable,
{
    subject_configs: Option<T::Config>,
    viewport: Option<Viewport>,
    axis: Option<ConfiguredElement<Axis>>,
    grid: Option<ConfiguredElement<GridLines>>,
    colorscheme: Option<Colorscheme>,
    ticks: Option<ConfiguredElement<TickLabels>>,
    title: Option<(String, TextStyle)>,
    xlabel: Option<(String, TextStyle)>,
    ylabel: Option<(String, TextStyle)>,
    legend: Option<Legend>,
    annotations: Vec<Annotation>,
}

impl<T> Default for GraphBuilder<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default + Themable,
{
    fn default() -> Self {
        Self {
            subject_configs: None,
            viewport: None,
            axis: None,
            grid: None,
            colorscheme: None,
            ticks: None,
            title: None,
            xlabel: None,
            ylabel: None,
            legend: None,
            annotations: Vec::new(),
        }
    }
}

impl<T> GraphBuilder<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default + Themable,
{
    // ── Original fields ──────────────────────────────────────────
    #[must_use]
    pub fn subject_configs(mut self, val: T::Config) -> Self {
        self.subject_configs = Some(val);
        self
    }

    #[must_use]
    pub fn viewport(mut self, val: Viewport) -> Self {
        self.viewport = Some(val);
        self
    }

    #[must_use]
    pub fn axis(mut self, val: impl Into<ConfiguredElement<Axis>>) -> Self {
        self.axis = Some(val.into());
        self
    }

    #[must_use]
    pub fn grid(mut self, val: impl Into<ConfiguredElement<GridLines>>) -> Self {
        self.grid = Some(val.into());
        self
    }

    #[must_use]
    pub fn colorscheme(mut self, val: Colorscheme) -> Self {
        self.colorscheme = Some(val);
        self
    }

    #[must_use]
    pub fn ticks(mut self, val: impl Into<ConfiguredElement<TickLabels>>) -> Self {
        self.ticks = Some(val.into());
        self
    }
    /// Set the chart title with sensible defaults (large, top-centre).
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn title(mut self, text: impl Into<String>) -> Self {
        let style = TextStyleBuilder::default()
            .font_size(26.0)
            .anchor(Anchor::TOP_CENTER)
            .build()
            .unwrap();
        self.title = Some((text.into(), style));
        self
    }

    /// Set the chart title with a customised style.
    ///
    /// The closure receives a `TextStyleBuilder` pre-configured with the
    /// default title settings; tweak only what you need.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn title_styled(
        mut self,
        text: impl Into<String>,
        f: impl FnOnce(TextStyleBuilder) -> TextStyleBuilder,
    ) -> Self {
        let base = TextStyleBuilder::default()
            .font_size(26.0)
            .anchor(Anchor::TOP_CENTER);
        let style = f(base).build().unwrap();
        self.title = Some((text.into(), style));
        self
    }

    /// Set the x-axis label with sensible defaults (centred below the plot).
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn xlabel(mut self, text: impl Into<String>) -> Self {
        let style = TextStyleBuilder::default()
            .font_size(20.0)
            .anchor(Anchor::CENTER_BOTTOM)
            .build()
            .unwrap();
        self.xlabel = Some((text.into(), style));
        self
    }

    /// Set the x-axis label with a customised style.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn xlabel_styled(
        mut self,
        text: impl Into<String>,
        f: impl FnOnce(TextStyleBuilder) -> TextStyleBuilder,
    ) -> Self {
        let base = TextStyleBuilder::default()
            .font_size(20.0)
            .anchor(Anchor::CENTER_BOTTOM);
        let style = f(base).build().unwrap();
        self.xlabel = Some((text.into(), style));
        self
    }

    /// Set the y-axis label with sensible defaults (centred, rotated -90°).
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn ylabel(mut self, text: impl Into<String>) -> Self {
        let style = TextStyleBuilder::default()
            .font_size(20.0)
            .anchor(Anchor::CENTER)
            .rotation(-90.0)
            .build()
            .unwrap();
        self.ylabel = Some((text.into(), style));
        self
    }

    /// Set the y-axis label with a customised style.
    #[must_use]
    #[allow(clippy::missing_panics_doc)]
    pub fn ylabel_styled(
        mut self,
        text: impl Into<String>,
        f: impl FnOnce(TextStyleBuilder) -> TextStyleBuilder,
    ) -> Self {
        let base = TextStyleBuilder::default()
            .font_size(20.0)
            .anchor(Anchor::CENTER)
            .rotation(-90.0);
        let style = f(base).build().unwrap();
        self.ylabel = Some((text.into(), style));
        self
    }

    /// Add a legend with default styling.
    #[must_use]
    pub fn legend(mut self, entries: Vec<LegendEntry>) -> Self {
        self.legend = Some(Legend {
            entries,
            ..Legend::default()
        });
        self
    }

    /// Add a legend with customised configuration.
    #[must_use]
    pub fn legend_styled(mut self, entries: Vec<LegendEntry>, f: impl FnOnce(&mut Legend)) -> Self {
        let mut leg = Legend {
            entries,
            ..Legend::default()
        };
        f(&mut leg);
        self.legend = Some(leg);
        self
    }

    /// Add a data-space annotation.
    #[must_use]
    pub fn annotate(
        mut self,
        text: impl Into<String>,
        data_point: impl Into<crate::plottable::point::Datapoint>,
    ) -> Self {
        let ann = Annotation::at_data(text, data_point);
        self.annotations.push(ann);
        self
    }

    /// Add a data-space annotation with customised style.
    #[must_use]
    pub fn annotate_styled(mut self, annotation: Annotation) -> Self {
        self.annotations.push(annotation);
        self
    }

    #[allow(clippy::missing_errors_doc)]
    pub fn build(self) -> Result<GraphConfig<T>, GraphBuilderError> {
        Ok(GraphConfig {
            subject_configs: self.subject_configs.unwrap_or_default(),
            viewport: self.viewport.unwrap_or_default(),
            axis: self.axis,
            grid: self.grid,
            colorscheme: self.colorscheme.unwrap_or_default(),
            ticks: self.ticks,
            title: self.title,
            xlabel: self.xlabel,
            ylabel: self.ylabel,
            legend: self.legend,
            annotations: self.annotations,
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
        if let Some((_, style)) = &mut self.title {
            style.apply_theme(&self.colorscheme);
        }
        if let Some((_, style)) = &mut self.xlabel {
            style.apply_theme(&self.colorscheme);
        }
        if let Some((_, style)) = &mut self.ylabel {
            style.apply_theme(&self.colorscheme);
        }
        if let Some(legend) = &mut self.legend {
            legend.apply_theme(&self.colorscheme);
        }
        for ann in &mut self.annotations {
            ann.apply_theme(&self.colorscheme);
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

        let outer = screen.outer_bbox();
        let inner = screen.inner_bbox();

        if let Some((text, style)) = &configs.title {
            // Centred horizontally at the top of the outer viewport, above the inner bbox.
            let origin = crate::plottable::point::Screenpoint::new(
                (inner.minimum.x + inner.maximum.x) * 0.5,
                (outer.minimum.y + inner.minimum.y) * 0.5,
            );
            style.draw(rl, text, origin);
        }

        if let Some((text, style)) = &configs.xlabel {
            // Centred horizontally below the inner bbox.
            let origin = crate::plottable::point::Screenpoint::new(
                (inner.minimum.x + inner.maximum.x) * 0.5,
                (outer.maximum.y + outer.maximum.y) * 0.5,
            );
            style.draw(rl, text, origin);
        }

        if let Some((text, style)) = &configs.ylabel {
            // Centred vertically to the left of the inner bbox.
            let origin = crate::plottable::point::Screenpoint::new(
                (inner.minimum.x + inner.minimum.x) * 0.5,
                (inner.minimum.y + inner.maximum.y) * 0.5,
            );
            style.draw(rl, text, origin);
        }

        if let Some(legend) = &configs.legend {
            legend.draw(rl, &inner);
        }

        for ann in &configs.annotations {
            ann.draw(rl, &view);
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
