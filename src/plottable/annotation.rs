#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]

use raylib::{color::Color, prelude::RaylibDraw};

use crate::{
    colorscheme::Themable,
    plottable::{
        point::{Datapoint, Screenpoint},
        text::{Anchor, TextStyle},
        view::ViewTransformer,
    },
};

// ── Annotation position ──────────────────────────────────────────────

/// Where the annotation text is placed.
#[derive(Debug, Clone, Copy)]
pub enum AnnotationPosition {
    /// Position in data coordinates — converted via `ViewTransformer`.
    Data(Datapoint),
    /// Position in screen coordinates.
    Screen(Screenpoint),
}

// ── Arrow configuration ──────────────────────────────────────────────

/// Visual properties for an optional leader line between the annotation
/// text and a data point.
#[derive(Debug, Clone, Copy)]
pub struct ArrowConfig {
    /// The data point the arrow points to.
    pub target: Datapoint,
    pub color: Option<Color>,
    pub thickness: f32,
}

impl ArrowConfig {
    #[must_use]
    pub fn new(target: impl Into<Datapoint>) -> Self {
        Self {
            target: target.into(),
            color: None,
            thickness: 1.5,
        }
    }
}

// ── Annotation ───────────────────────────────────────────────────────

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
    pub style: TextStyle,
    pub arrow: Option<ArrowConfig>,
}

impl Annotation {
    /// Create an annotation at a data-space position.
    #[must_use]
    pub fn at_data(text: impl Into<String>, point: impl Into<Datapoint>) -> Self {
        Self {
            text: text.into(),
            position: AnnotationPosition::Data(point.into()),
            style: TextStyle {
                font_size: 14.0,
                anchor: Anchor::CENTER_BOTTOM,
                ..TextStyle::default()
            },
            arrow: None,
        }
    }

    /// Create an annotation at a fixed screen-space position.
    #[must_use]
    pub fn at_screen(text: impl Into<String>, point: impl Into<Screenpoint>) -> Self {
        Self {
            text: text.into(),
            position: AnnotationPosition::Screen(point.into()),
            style: TextStyle {
                font_size: 14.0,
                anchor: Anchor::CENTER_BOTTOM,
                ..TextStyle::default()
            },
            arrow: None,
        }
    }

    /// Attach a leader-line arrow pointing to a data point.
    #[must_use]
    pub fn with_arrow(mut self, arrow: ArrowConfig) -> Self {
        self.arrow = Some(arrow);
        self
    }

    /// Replace the text style.
    #[must_use]
    pub fn with_style(mut self, style: TextStyle) -> Self {
        self.style = style;
        self
    }

    /// Draw the annotation.  Needs a `ViewTransformer` to resolve data positions.
    pub fn draw(
        &self,
        rl: &mut raylib::prelude::RaylibDrawHandle,
        view: &ViewTransformer,
    ) {
        let origin = match self.position {
            AnnotationPosition::Data(dp) => view.to_screen(&dp),
            AnnotationPosition::Screen(sp) => sp,
        };

        // Draw leader line first (under text).
        if let Some(arrow) = &self.arrow {
            let target_screen = view.to_screen(&arrow.target);
            let color = arrow.color.unwrap_or_else(|| self.style.effective_color());
            rl.draw_line_ex(*origin, *target_screen, arrow.thickness, color);
        }

        self.style.draw(rl, &self.text, origin);
    }
}

impl Themable for Annotation {
    fn apply_theme(&mut self, scheme: &crate::colorscheme::Colorscheme) {
        self.style.apply_theme(scheme);
        if let Some(arrow) = &mut self.arrow
            && arrow.color.is_none()
        {
            arrow.color = Some(scheme.text);
        }
    }
}
