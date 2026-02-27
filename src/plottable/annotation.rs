//! Data-space text annotations with optional leader arrows.
//!
//! An [`Annotation`] places a text label at a specific location in either
//! data or screen coordinates. When combined with an [`AnnotLineConfig`],
//! a leader line (optionally with an arrowhead) is drawn from the label
//! origin to a target data point, making it easy to call out specific
//! features in a plot.
//!
//! Annotations are added to a graph through
//! [`GraphBuilder::annotate`](crate::graph::GraphBuilder::annotate) or
//! [`GraphBuilder::annotate_styled`](crate::graph::GraphBuilder::annotate_styled).
//!
//! # Example
//!
//! ```rust
//! use locus::prelude::*;
//!
//! // Simple annotation at a data point
//! let ann = Annotation::at_data("outlier", (3.5, 12.0));
//!
//! // With a styled leader arrow
//! # let mut builder: GraphBuilder<ScatterPlot> = GraphBuilder::default();
//! builder.annotate_styled(ann, |c| {
//!     c.line = Some(
//!         AnnotLineConfigBuilder::default()
//!             .target((3.5, 10.0).into())
//!             .arrow(Visibility::Visible)
//!             .build()
//!             .unwrap(),
//!     );
//! });
//! ```

use derive_builder::Builder;
use raylib::color::Color;

use crate::{
    TextLabel,
    colorscheme::Themable,
    plottable::{
        line::{Line, LineConfigBuilder, Visibility},
        point::{Datapoint, Screenpoint},
        text::TextStyle,
        view::ViewTransformer,
    },
    plotter::{ChartElement, PlotElement},
};

/// Where the annotation text is placed.
#[derive(Debug, Clone, Copy)]
pub enum AnnotationPosition {
    /// Position in data coordinates — converted via `ViewTransformer`.
    Data(Datapoint),
    /// Position in screen coordinates.
    Screen(Screenpoint),
}

/// A text annotation placed at a specific location;
///
#[derive(Debug, Clone)]
pub struct Annotation {
    pub text: String,
    pub position: AnnotationPosition,
}

/// Configuration for the leader line drawn from an annotation to a target
/// data point.
#[derive(Clone, Debug, Builder)]
#[builder(pattern = "owned")]
pub struct AnnotLineConfig {
    /// Line thickness in pixels.
    #[builder(default = "1.5")]
    pub thickness: f32,
    /// Explicit line color. `None` is resolved from the theme.
    #[builder(setter(into, strip_option), default = "None")]
    pub color: Option<Color>,
    /// Whether to draw an arrowhead at the target end.
    #[builder(default = "Visibility::Visible")]
    pub arrow: Visibility,
    /// Length of the arrowhead along the line direction (pixels).
    #[builder(default = "4.0 * 1.5")]
    pub arrow_length: f32,
    /// Half-width of the arrowhead perpendicular to the line (pixels).
    #[builder(default = "3.5 * 1.5")]
    pub arrow_width: f32,
    /// The data-space point that the leader line points toward.
    pub target: Datapoint,
}

/// Configuration for an [`Annotation`], controlling text style and the
/// optional leader line.
#[derive(Debug, Clone, Builder, Default)]
#[builder(pattern = "owned")]
pub struct AnnotationConfig {
    /// Visual style of the annotation text.
    #[builder(default = "TextStyle::default()")]
    pub style: TextStyle,
    /// Optional leader line configuration. When `Some`, a line is drawn from
    /// the annotation origin to the specified target data point.
    #[builder(setter(into, strip_option), default = "None")]
    pub line: Option<AnnotLineConfig>,
}

impl AnnotationConfigBuilder {
    fn with_target(self, target: impl Into<Datapoint>) -> Self {
        if let Some(Some(line)) = self.line {
            Self {
                line: Some(Some(AnnotLineConfig {
                    target: target.into(),
                    ..line
                })),
                ..self
            }
        } else {
            Self {
                line: Some(Some(AnnotLineConfig {
                    target: target.into(),
                    thickness: 1.5,
                    color: None,
                    arrow: Visibility::Visible,
                    arrow_length: 1.5,
                    arrow_width: 1.5,
                })),
                ..self
            }
        }
    }
}

impl Annotation {
    /// Create an annotation at a data-space position.
    #[must_use]
    pub fn at_data(text: impl Into<String>, point: impl Into<Datapoint>) -> Self {
        Self {
            text: text.into(),
            position: AnnotationPosition::Data(point.into()),
        }
    }

    /// Create an annotation at a fixed screen-space position.
    #[must_use]
    pub fn at_screen(text: impl Into<String>, point: impl Into<Screenpoint>) -> Self {
        Self {
            text: text.into(),
            position: AnnotationPosition::Screen(point.into()),
        }
    }
}

impl ChartElement for Annotation {
    type Config = AnnotationConfig;

    fn draw_in_view(
        &self,
        rl: &mut raylib::prelude::RaylibDrawHandle,
        configs: &Self::Config,
        view: &ViewTransformer,
    ) {
        let origin = match self.position {
            AnnotationPosition::Data(dp) => view.to_screen(&dp),
            AnnotationPosition::Screen(sp) => sp,
        };

        // Draw leader line first (under text).
        if let Some(annot_line_configs) = &configs.line {
            let target_screen = view.to_screen(&annot_line_configs.target);
            let line = Line::new(*origin, *target_screen);
            let mut line_configs = LineConfigBuilder::default()
                .arrow_width(annot_line_configs.arrow_width)
                .thickness(annot_line_configs.thickness)
                .arrow_length(annot_line_configs.arrow_length)
                .arrow(annot_line_configs.arrow)
                .build()
                .unwrap();
            line_configs.color = annot_line_configs.color;
            line.plot(rl, &line_configs);
        }
        let text = TextLabel::new(&self.text, origin);
        text.plot(rl, &configs.style);
    }

    fn data_bounds(&self) -> super::view::DataBBox {
        unimplemented!("Doesn't make sense for annotation")
    }
}

impl Themable for AnnotationConfig {
    fn apply_theme(&mut self, scheme: &crate::colorscheme::Colorscheme) {
        self.style.apply_theme(scheme);
        if let Some(line_configs) = &mut self.line
            && line_configs.color.is_none()
            && self.style.color.is_none()
        {
            line_configs.color = Some(scheme.text);
        }
        if let Some(line_configs) = &mut self.line
            && line_configs.color.is_none()
            && self.style.color.is_some()
        {
            line_configs.color = self.style.color;
        }
    }
}
