#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]

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

/// A text annotation placed at a specific location, optionally with a
/// leader line to a data point.
///
/// ```ignore
/// Annotation::new("outlier", (3.5, 12.0))
///     .with_arrow(ArrowConfig::new((3.5, 10.0)));
/// ```
#[derive(Debug, Clone)]
pub struct Annotation {
    pub text: String,
    pub position: AnnotationPosition,
}

#[derive(Clone, Debug, Builder)]
#[builder(pattern = "owned")]
pub struct AnnotLineConfig {
    #[builder(default = "1.5")]
    pub thickness: f32,
    #[builder(setter(into, strip_option), default = "None")]
    pub color: Option<Color>, // Defaults means use theme's
    #[builder(default = "Visibility::Visible")]
    pub arrow: Visibility,
    #[builder(default = "4.0 * 1.5")]
    pub arrow_length: f32, // Only matters if arrow is visible
    #[builder(default = "3.5 * 1.5")]
    pub arrow_width: f32, // Only matters if arrow is visible
    pub target: Datapoint,
}

#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned")]
pub struct AnnotationConfig {
    #[builder(default = "TextStyle::default()")]
    pub style: TextStyle,
    #[builder(setter(into, strip_option), default = "None")]
    pub line: Option<AnnotLineConfig>,
}

impl Default for AnnotationConfig {
    fn default() -> Self {
        Self {
            style: TextStyle::default(),
            line: None,
        }
    }
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

// #[derive(Clone, Builder)]
// #[builder(pattern = "owned")]
// pub struct AnnotationConfigs {
//     #[builder(default = "TextStyle::default()")]
//     #[builder(default = "None", setter(strip_option))]
// }

impl Annotation {
    /// Create an annotation at a data-space position.
    #[must_use]
    pub fn at_data(text: impl Into<String>, point: impl Into<Datapoint>) -> Self {
        Self {
            text: text.into(),
            position: AnnotationPosition::Data(point.into()),
            // style: TextStyle {
            //     font_size: 14.0,
            //     alpha: 1.0,
            //     color: Some(Color::WHITE),
            //     spacing: 1.0,
            //     anchor: Anchor::CENTER,
            //     ..TextStyle::default()
            // },
            // line: None,
        }
    }

    /// Create an annotation at a fixed screen-space position.
    #[must_use]
    pub fn at_screen(text: impl Into<String>, point: impl Into<Screenpoint>) -> Self {
        Self {
            text: text.into(),
            position: AnnotationPosition::Screen(point.into()),
            // style: TextStyle {
            //     font_size: 14.0,
            //     alpha: 1.0,
            //     color: Some(Color::WHITE),
            //     spacing: 1.0,
            //     ..TextStyle::default()
            // },
            // line: None,
        }
    }
    // #[must_use]
    // pub fn with_line(mut self, line: LineConfig) -> Self {
    //     self.line = Some(line);
    //     self
    // }

    // #[must_use]
    // pub fn with_style(mut self, style: TextStyle) -> Self {
    //     self.style = style;
    //     self
    // }
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
            && !self.style.color.is_none()
        {
            line_configs.color = self.style.color;
        }
    }
}
