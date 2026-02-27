//! The main graph orchestrator and its builder.
//!
//! [`Graph`] wraps any [`ChartElement`] and manages the surrounding chrome:
//! axes, grid lines, tick marks, labels, legends, annotations, and the color
//! theme. At render time it constructs a [`ViewTransformer`] from the data
//! bounds and the configured viewport, then delegates drawing to each
//! sub-element.
//!
//! Configuration is assembled via the [`GraphBuilder`], which follows the
//! builder pattern and provides convenience methods for common tasks such as
//! setting a title, axis labels, or adding a legend.
//!
//! # Example
//!
//! ```rust,no_run
//! use locus::prelude::*;
//! # let axis = Axis::fitting(0.0..10.0,0.0..10.0, 1.0,10);
//! # let grid = GridLines::new(axis, Orientation::default());
//! # let ticks = TickLabels::new(axis);
//! # let my_scheme = DRACULA.clone();
//! # let dataset = Dataset::new(vec![(0.0,0.0), (1.0,1.0), (2.0, 2.0)]);
//! # let scatter_plot = ScatterPlot::new(&dataset);
//! # let (mut rl, rl_thread) = raylib::init()
//! #       .width(WIDTH)
//! #       .height(HEIGHT)
//! #       .title("Datasets")
//! #       .build();
//! # let mut draw_handle = rl.begin_drawing(&rl_thread);
//! # const IMAGE_SIZE: i32 = 90;
//! # const WIDTH: i32 = 16 * IMAGE_SIZE;
//! # const HEIGHT: i32 = 9 * IMAGE_SIZE;
//! let graph = Graph::new(scatter_plot);
//! let config = GraphBuilder::default()
//!     .viewport(
//!         Viewport::new(0.0, 0.0, 800.0, 600.0)
//!             .with_margins(Margins::all(50.0)),
//!     )
//!     .axis(ConfiguredElement::with_defaults(axis))
//!     .grid(ConfiguredElement::with_defaults(grid))
//!     .ticks(ConfiguredElement::with_defaults(ticks))
//!     .title("My Plot")
//!     .xlabel("X")
//!     .ylabel("Y")
//!     .colorscheme(my_scheme)
//!     .build()
//!     .unwrap();
//! {
//! // inside the render loop:
//! graph.plot(&mut draw_handle, &config);
//! }
//! ```

use crate::{
    TextLabel,
    colorscheme::{Colorscheme, Themable},
    plottable::{
        annotation::{Annotation, AnnotationConfig},
        legend::{Legend, LegendConfig, LegendEntry},
        line::{Axis, AxisConfigs, GridLines, GridLinesConfig, TickLabels, TickLabelsConfig},
        point::Datapoint,
        text::{Anchor, TextStyle, TextStyleBuilder},
        view::{ScreenBBox, ViewTransformer, Viewport},
    },
    plotter::{ChartElement, PlotElement},
};
use raylib::prelude::RaylibScissorModeExt;
/// Represents a graph over `subject`, orchestrating elements such as axes,
/// grid lines, tick marks, labels, legends, and annotations.
///
/// `Graph` implements [`PlotElement`] so the fully assembled visualization can
/// be drawn with a single call to [`plot`](PlotElement::plot). Internally it
/// constructs a [`ViewTransformer`] from the subject's data bounds (or the
/// explicit axis bounds) and the configured [`Viewport`], then renders each
/// sub-element in the correct order (background grid, data, axes, ticks,
/// labels, legend, annotations).
///
/// Configuration is provided through [`GraphConfig`], which is most
/// conveniently built via [`GraphBuilder`].
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
    /// Wrap a chart element so it can be rendered as a complete graph.
    pub fn new(subject: T) -> Self {
        Self { subject }
    }
}

/// A visual element paired with its configuration.
///
/// `ConfiguredElement` binds any drawable element (`E`) to the configuration
/// struct (`C`) it needs at draw time. This coupling is used throughout the
/// graph to store optional chrome elements (axis, grid, ticks, labels, etc.)
/// together with the settings chosen by the user.
///
/// The struct exposes [`draw_in_view`](ConfiguredElement::draw_in_view) for
/// data-space elements and [`draw`](ConfiguredElement::draw) for screen-space
/// elements, forwarding to the underlying trait implementations.
#[derive(Debug, Clone)]
pub struct ConfiguredElement<E, C> {
    pub(crate) element: E,
    pub(crate) configs: C,
}

impl<E, C> ConfiguredElement<E, C>
where
    E: ChartElement,
    E: ChartElement<Config = C>,
{
    /// Create a configured element from an element and its configuration.
    pub fn new(element: E, configs: C) -> Self {
        Self { element, configs }
    }
    /// Draw this element in data space, projecting through `view`.
    pub fn draw_in_view(&self, rl: &mut raylib::prelude::RaylibDrawHandle, view: &ViewTransformer) {
        self.element.draw_in_view(rl, &self.configs, view);
    }
}

impl<E, C> ConfiguredElement<E, C>
where
    E: PlotElement,
    E: PlotElement<Config = C>,
{
    /// Draw this element directly in screen space.
    pub fn draw(&self, rl: &mut raylib::prelude::RaylibDrawHandle) {
        self.element.plot(rl, &self.configs);
    }
}

impl<E, C> Themable for ConfiguredElement<E, C>
where
    C: Themable,
{
    fn apply_theme(&mut self, scheme: &Colorscheme) {
        self.configs.apply_theme(scheme);
    }
}

impl<E, C> ConfiguredElement<E, C>
where
    C: Default,
{
    /// Create with default configuration.
    pub fn with_defaults(element: E) -> Self {
        Self {
            element,
            configs: C::default(),
        }
    }
}

impl<E, C> ConfiguredElement<E, C> {
    /// Modify the config via a closure, returning self for chaining.
    #[must_use]
    pub fn configure(mut self, f: impl FnOnce(&mut C)) -> Self {
        f(&mut self.configs);
        self
    }
}

/// Complete, resolved configuration for a [`Graph`].
///
/// A `GraphConfig` holds all optional chrome elements (axis, grid, ticks,
/// title, axis labels, legend, annotations) together with the subject's own
/// configuration and the active [`Colorscheme`]. After construction via
/// [`GraphBuilder::build`] the theme is automatically resolved so that every
/// `None` color field is filled from the scheme.
///
/// Because resolving the theme is a pure function of the config, callers
/// should build the config once (outside the render loop) and reuse it
/// every frame.
#[derive(Debug, Clone)]
pub struct GraphConfig<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default + Themable,
{
    subject_configs: T::Config,
    viewport: Viewport,
    axis: Option<ConfiguredElement<Axis, AxisConfigs>>,
    grid: Option<ConfiguredElement<GridLines, GridLinesConfig>>,
    colorscheme: Colorscheme,
    ticks: Option<ConfiguredElement<TickLabels, TickLabelsConfig>>,
    title: Option<ConfiguredElement<TextLabel, TextStyle>>,
    xlabel: Option<ConfiguredElement<TextLabel, TextStyle>>,
    ylabel: Option<ConfiguredElement<TextLabel, TextStyle>>,
    legend: Option<ConfiguredElement<Legend, LegendConfig>>,
    annotations: Option<Vec<ConfiguredElement<Annotation, AnnotationConfig>>>,
}

/// Error returned when [`GraphBuilder::build`] fails due to missing or
/// inconsistent configuration.
#[derive(Debug, Clone)]
pub struct GraphBuilderError(String);

impl std::fmt::Display for GraphBuilderError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "GraphBuilderError: {}", self.0)
    }
}

impl std::error::Error for GraphBuilderError {}

/// Ergonomic builder for [`GraphConfig`].
///
/// All fields are optional. Omitted chrome (axis, grid, ticks, title, etc.)
/// is simply not drawn. The builder provides both plain and `_styled`
/// variants for text elements so that simple cases remain one-liners while
/// full customisation is still possible.
///
/// # Example
///
/// ```rust,no_run
/// # use locus::prelude::*;
/// # use raylib::color::Color;
/// # const IMAGE_SIZE: i32 = 90;
/// # const WIDTH: i32 = 16 * IMAGE_SIZE;
/// # const HEIGHT: i32 = 9 * IMAGE_SIZE;
/// # let axis = Axis::fitting(0.0..10.0,0.0..10.0, 1.0,10);
/// # let grid = GridLines::new(axis, Orientation::default());
/// # let ticks = TickLabels::new(axis);
/// # let scheme = DRACULA.clone();
/// # let (mut rl, rl_thread) = raylib::init()
/// #       .width(WIDTH)
/// #       .height(HEIGHT)
/// #       .title("Datasets")
/// #       .build();
/// # let mut draw_handle = rl.begin_drawing(&rl_thread);
/// # let vp = Viewport::new(10.0, 10.0, (WIDTH / 2) as f32, (HEIGHT - 15) as f32)
/// #                        .with_margins(Margins {
/// #                            left: 40.0,
/// #                            right: 10.0,
/// #                            top: 10.0,
/// #                            bottom: 30.0,
/// #                        });
/// # let entries = vec![
/// #                    LegendEntry::new("Sample points", Color::RED),
/// #                    LegendEntry::new("Cluster A", Color::SKYBLUE).with_shape(Shape::Rectangle),
/// #                    LegendEntry::new("Cluster B", Color::GREEN).with_shape(Shape::Triangle),
/// #                ];
/// # let my_configs = ScatterPlotBuilder::default().build().unwrap();
/// # let dataset = Dataset::new(vec![(0.0,0.0), (1.0,1.0), (2.0, 2.0)]);
/// # let scatter_plot = ScatterPlot::new(&dataset);
/// let graph = Graph::new(scatter_plot);
/// let config = GraphBuilder::default()
///     .viewport(vp)
///     .axis(ConfiguredElement::with_defaults(axis))
///     .grid(ConfiguredElement::with_defaults(grid))
///     .ticks(ConfiguredElement::with_defaults(ticks))
///     .title("My Chart")
///     .xlabel("X")
///     .ylabel("Y")
///     .colorscheme(scheme)
///     .legend(entries)
///     .subject_configs(my_configs)
///     .build()
///     .unwrap();
/// # graph.plot(&mut draw_handle, &config);
/// ```
pub struct GraphBuilder<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default + Themable,
{
    subject_configs: Option<T::Config>,
    viewport: Option<Viewport>,
    axis: Option<ConfiguredElement<Axis, AxisConfigs>>,
    grid: Option<ConfiguredElement<GridLines, GridLinesConfig>>,
    colorscheme: Option<Colorscheme>,
    ticks: Option<ConfiguredElement<TickLabels, TickLabelsConfig>>,
    title: Option<(String, TextStyle)>,
    xlabel: Option<(String, TextStyle)>,
    ylabel: Option<(String, TextStyle)>,
    legend: Option<ConfiguredElement<Legend, LegendConfig>>,
    annotations: Option<Vec<ConfiguredElement<Annotation, AnnotationConfig>>>,
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
            annotations: None,
        }
    }
}

impl<T> GraphBuilder<T>
where
    T: ChartElement,
    <T as ChartElement>::Config: Default + Themable,
{
    /// Set the subject-specific configuration (e.g. [`ScatterPlotConfig`](crate::plottable::scatter::ScatterPlotConfig)).
    #[must_use]
    pub fn subject_configs(mut self, val: T::Config) -> Self {
        self.subject_configs = Some(val);
        self
    }

    /// Set the screen-space region and margins where the graph is rendered.
    #[must_use]
    pub fn viewport(mut self, val: Viewport) -> Self {
        self.viewport = Some(val);
        self
    }

    /// Add axis lines to the graph.
    #[must_use]
    pub fn axis(mut self, val: impl Into<ConfiguredElement<Axis, AxisConfigs>>) -> Self {
        self.axis = Some(val.into());
        self
    }

    /// Add grid lines to the graph.
    #[must_use]
    pub fn grid(mut self, val: impl Into<ConfiguredElement<GridLines, GridLinesConfig>>) -> Self {
        self.grid = Some(val.into());
        self
    }

    /// Set the color scheme used to resolve theme-dependent defaults.
    #[must_use]
    pub fn colorscheme(mut self, val: Colorscheme) -> Self {
        self.colorscheme = Some(val);
        self
    }

    /// Add tick marks and optional tick labels along the axes.
    #[must_use]
    pub fn ticks(
        mut self,
        val: impl Into<ConfiguredElement<TickLabels, TickLabelsConfig>>,
    ) -> Self {
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
        let legend = Legend { entries };
        let element = ConfiguredElement::new(legend, LegendConfig::default());
        self.legend = Some(element);
        self
    }

    /// Add a legend with customised configuration.
    #[must_use]
    pub fn legend_styled(
        mut self,
        entries: Vec<LegendEntry>,
        f: impl FnOnce(&mut LegendConfig),
    ) -> Self {
        let legend = Legend { entries };
        let mut config = LegendConfig::default();
        f(&mut config);
        self.legend = Some(ConfiguredElement::new(legend, config));
        self
    }

    /// Add a data-space annotation.
    #[must_use]
    pub fn annotate(mut self, text: impl Into<String>, data_point: impl Into<Datapoint>) -> Self {
        let annotation = Annotation::at_data(text, data_point);

        if self.annotations.is_none() {
            self.annotations = Some(Vec::new());
        }
        if let Some(annot) = self.annotations.as_mut() {
            annot.push(ConfiguredElement {
                element: annotation,
                configs: AnnotationConfig::default(),
            });
        }
        self
    }

    /// Add a data-space annotation with customised style.
    #[must_use]
    pub fn annotate_styled(
        mut self,
        annotation: Annotation,
        f: impl FnOnce(&mut AnnotationConfig),
    ) -> Self {
        let mut configs = AnnotationConfig::default();
        f(&mut configs);
        if self.annotations.is_none() {
            self.annotations = Some(Vec::new());
        }
        if let Some(annot) = self.annotations.as_mut() {
            annot.push(ConfiguredElement {
                element: annotation,
                configs,
            });
        }
        self
    }

    /// Consume the builder and produce a fully resolved [`GraphConfig`].
    ///
    /// Returns an error if required fields are missing or inconsistent.
    /// On success the returned config has all theme-dependent colors resolved,
    /// making it safe to reuse across frames without further mutation.
    #[allow(clippy::missing_errors_doc)]
    pub fn build(self) -> Result<GraphConfig<T>, GraphBuilderError> {
        let viewport = self.viewport.unwrap_or_default();
        let inner = viewport.inner_bbox();
        let outer = viewport.outer_bbox();
        let title: Option<ConfiguredElement<TextLabel, TextStyle>> =
            if let Some((text, configs)) = self.title {
                // Centred horizontally at the top of the outer viewport, above the inner bbox.
                let origin = crate::plottable::point::Screenpoint::new(
                    (inner.minimum.x + inner.maximum.x) * 0.5,
                    (outer.minimum.y + inner.minimum.y) * 0.5,
                );
                let element = TextLabel::new(text, origin);
                Some(ConfiguredElement { element, configs })
            } else {
                None
            };

        let xlabel: Option<ConfiguredElement<TextLabel, TextStyle>> =
            if let Some((text, configs)) = self.xlabel {
                // Centred horizontally below the inner bbox.
                let origin = crate::plottable::point::Screenpoint::new(
                    (inner.minimum.x + inner.maximum.x) * 0.5,
                    (outer.maximum.y + outer.maximum.y) * 0.5,
                );
                let element = TextLabel::new(text, origin);
                Some(ConfiguredElement { element, configs })
            } else {
                None
            };
        let ylabel: Option<ConfiguredElement<TextLabel, TextStyle>> =
            if let Some((text, configs)) = self.ylabel {
                // Centred vertically to the left of the inner bbox.
                let origin = crate::plottable::point::Screenpoint::new(
                    (inner.minimum.x + inner.minimum.x) * 0.5,
                    (inner.minimum.y + inner.maximum.y) * 0.5,
                );
                let element = TextLabel::new(text, origin);
                Some(ConfiguredElement { element, configs })
            } else {
                None
            };
        Ok(GraphConfig {
            subject_configs: self.subject_configs.unwrap_or_default(),
            viewport: self.viewport.unwrap_or_default(),
            axis: self.axis,
            grid: self.grid,
            colorscheme: self.colorscheme.unwrap_or_default(),
            ticks: self.ticks,
            title,
            xlabel,
            ylabel,
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
        if let Some(title) = &mut self.title {
            title.apply_theme(&self.colorscheme);
        }
        if let Some(xlabel) = &mut self.xlabel {
            xlabel.apply_theme(&self.colorscheme);
        }
        if let Some(ylabel) = &mut self.ylabel {
            ylabel.apply_theme(&self.colorscheme);
        }
        if let Some(legend) = &mut self.legend {
            legend.apply_theme(&self.colorscheme);
        }
        if let Some(annotations) = &mut self.annotations {
            for ann in annotations {
                ann.apply_theme(&self.colorscheme);
            }
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
                grid.draw_in_view(&mut scissors, &view);
            }

            // We plot the subject inside the view.
            // configs.subject_configs.apply_theme(&configs.colorscheme);
            self.subject
                .draw_in_view(&mut scissors, &configs.subject_configs, &view);
        }
        // NOTE: Axis shouldn't be scissored, neither the ticks;
        if let Some(axis) = &configs.axis {
            axis.draw_in_view(rl, &view);
        }
        if let Some(ticks) = &configs.ticks {
            ticks.draw_in_view(rl, &view);
        }

        if let Some(title) = &configs.title {
            title.draw(rl);
        }
        if let Some(xlabel) = &configs.title {
            xlabel.draw(rl);
        }
        if let Some(ylabel) = &configs.title {
            ylabel.draw(rl);
        }

        if let Some(legend) = &configs.legend {
            legend.draw_in_view(rl, &view);
        }
        if let Some(annotations) = &configs.annotations {
            for annot in annotations {
                annot.draw_in_view(rl, &view);
            }
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
