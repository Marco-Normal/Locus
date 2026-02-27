//! Configurable legend box with color swatches and text labels.
//!
//! A [`Legend`] is a list of [`LegendEntry`] items rendered inside the graph
//! viewport. Each entry shows a colored shape indicator next to a text label,
//! making it easy for viewers to identify data series.
//!
//! Legends are added to a graph through
//! [`GraphBuilder::legend`](crate::graph::GraphBuilder::legend) or
//! [`GraphBuilder::legend_styled`](crate::graph::GraphBuilder::legend_styled).
//!
//! # Example
//!
//! ```rust
//! use locus::prelude::*;
//! use raylib::color::Color;
//! # let mut builder: GraphBuilder<ScatterPlot> = GraphBuilder::default();
//!
//! let entries = vec![
//!     LegendEntry::new("Cluster A", Color::RED),
//!     LegendEntry::new("Cluster B", Color::BLUE).with_shape(Shape::Triangle),
//! ];
//! builder.legend(entries);
//! ```

use derive_builder::Builder;
use raylib::{
    color::Color,
    math::{Rectangle, Vector2},
    prelude::RaylibDraw,
    text::WeakFont,
};

use crate::{
    Anchor, TextLabel,
    colorscheme::Themable,
    plottable::{
        point::{Screenpoint, Shape},
        text::{TextStyle, TextStyleBuilder},
    },
    plotter::{ChartElement, PlotElement},
};

/// Where to anchor the legend box relative to the inner plotting area.
#[derive(Debug, Clone, Copy, Default)]
pub enum LegendPosition {
    /// Upper-right corner of the inner plotting area (the default).
    #[default]
    TopRight,
    /// Upper-left corner.
    TopLeft,
    /// Lower-right corner.
    BottomRight,
    /// Lower-left corner.
    BottomLeft,
    /// Arbitrary screen-space coordinates for the top-left corner of the box.
    Custom(f32, f32),
}

/// A single entry in a legend: a color swatch, indicator shape, and label.
#[derive(Debug, Clone)]
pub struct LegendEntry {
    /// Display text for this entry.
    pub label: String,
    /// Color of the shape indicator.
    pub color: Color,
    /// Shape used for the indicator swatch.
    pub shape: Shape,
}

impl LegendEntry {
    /// Create a legend entry with the given label and color, defaulting to a
    /// circle indicator.
    #[must_use]
    pub fn new(label: impl Into<String>, color: Color) -> Self {
        Self {
            label: label.into(),
            color,
            shape: Shape::Circle,
        }
    }

    /// Override the default circle indicator with a different shape.
    #[must_use]
    pub fn with_shape(mut self, shape: Shape) -> Self {
        self.shape = shape;
        self
    }
}

/// A drawable legend that pairs colour swatches with text labels.
///
/// Constructed via `LegendBuilder` and added to a `Graph` with
/// `.legend(entries)` or `.legend_styled(entries, |c| ...)`.
#[derive(Default, Clone, Debug)]
pub struct Legend {
    pub entries: Vec<LegendEntry>,
}

/// Configuration for the [`Legend`] box appearance and layout.
#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned")]
pub struct LegendConfig {
    /// Positioning anchor for the legend box.
    #[builder(default)]
    pub position: LegendPosition,
    /// Text style for entry labels.
    #[builder(default)]
    pub label_style: TextStyle,
    /// Semi-transparent background color behind the legend box. Set to
    /// `None` to draw without a background.
    #[builder(default = "Some(Color::new(0, 0, 0, 140))")]
    pub background: Option<Color>,
    /// Padding inside the background box in pixels.
    #[builder(default = "8.0")]
    pub padding: f32,
    /// Vertical spacing between consecutive entries in pixels.
    #[builder(default = "4.0")]
    pub entry_spacing: f32,
    /// Size of the color swatch indicator in pixels.
    #[builder(default = "8.0")]
    pub indicator_size: f32,
    /// Gap between the swatch and the label text in pixels.
    #[builder(default = "6.0")]
    pub indicator_gap: f32,
    /// Optional border as `(color, thickness)`. `None` means no border.
    #[builder(default = "None")]
    pub border: Option<(Color, f32)>,
}

impl Default for LegendConfig {
    fn default() -> Self {
        Self {
            position: LegendPosition::default(),
            label_style: TextStyleBuilder::default()
                .font_size(14.0)
                .anchor(Anchor::TOP_LEFT)
                .build()
                .unwrap(),
            background: Some(Color {
                r: 0,
                g: 0,
                b: 0,
                a: 140,
            }),
            padding: 8.0,
            entry_spacing: 4.0,
            indicator_size: 8.0,
            indicator_gap: 6.0,
            border: None,
        }
    }
}

impl ChartElement for Legend {
    type Config = LegendConfig;
    #[allow(clippy::cast_precision_loss, clippy::cast_possible_truncation)]
    fn draw_in_view(
        &self,
        rl: &mut raylib::prelude::RaylibDrawHandle,
        configs: &Self::Config,
        view: &super::view::ViewTransformer,
    ) {
        if self.entries.is_empty() {
            return;
        }

        let font: &WeakFont = match &configs.label_style.font {
            Some(fh) => &fh.font,
            None => &rl.get_font_default(),
        };

        let row_height = configs.label_style.font_size;
        let n = self.entries.len();
        let total_height = configs.padding * 2.0
            + (n as f32) * row_height
            + ((n.saturating_sub(1)) as f32) * configs.entry_spacing;
        let mut max_label_width: f32 = 0.0;
        for entry in &self.entries {
            let size = configs.label_style.measure_text(&entry.label, font);
            max_label_width = max_label_width.max(size.x);
        }

        let total_width = configs.padding * 2.0
            + configs.indicator_size
            + configs.indicator_gap
            + max_label_width;

        let inner_bbox = view.screen_bounds.inner_bbox();

        let legend_box: Vector2 = match configs.position {
            LegendPosition::TopRight => {
                (inner_bbox.maximum.x - total_width, inner_bbox.minimum.y).into()
            }
            LegendPosition::TopLeft => (inner_bbox.minimum.x, inner_bbox.minimum.y).into(),
            LegendPosition::BottomRight => (
                inner_bbox.maximum.x - total_width,
                inner_bbox.maximum.y - total_height,
            )
                .into(),
            LegendPosition::BottomLeft => {
                (inner_bbox.minimum.x, inner_bbox.maximum.y - total_height).into()
            }
            LegendPosition::Custom(x, y) => (x, y).into(),
        };

        if let Some(bg) = configs.background {
            rl.draw_rectangle_v(legend_box, Vector2::new(total_width, total_height), bg);
        }
        if let Some((border_color, thickness)) = configs.border {
            rl.draw_rectangle_lines_ex(
                Rectangle {
                    x: legend_box.x,
                    y: legend_box.y,
                    width: total_width,
                    height: total_height,
                },
                thickness,
                border_color,
            );
        }

        for (i, entry) in self.entries.iter().enumerate() {
            let row_y =
                legend_box.y + configs.padding + (i as f32) * (row_height + configs.entry_spacing);
            let swatch_x = legend_box.x + configs.padding;
            let swatch_cy = row_y + row_height * 0.5;
            // NOTE: Whilst we do have a point primitive where we could use it to draw the shapes, it doesn't
            // fit the best because of how the icons should be placed. It would be best to unify the API, as
            // the inclusion of more shapes could be reflected automatically in the legend, instead of having
            // double code. As of right now, this is somewhat ok.
            // TODO: Maybe unify to use the point primitive for icon drawing
            match entry.shape {
                Shape::Circle => {
                    rl.draw_circle(
                        swatch_x as i32 + (configs.indicator_size * 0.5) as i32,
                        swatch_cy as i32,
                        configs.indicator_size * 0.5,
                        entry.color,
                    );
                }
                Shape::Rectangle => {
                    rl.draw_rectangle_v(
                        Vector2::new(swatch_x, swatch_cy - configs.indicator_size * 0.5),
                        Vector2::new(configs.indicator_size, configs.indicator_size),
                        entry.color,
                    );
                }
                Shape::Triangle => {
                    let cx = swatch_x + configs.indicator_size * 0.5;
                    let half = configs.indicator_size * 0.5;
                    rl.draw_triangle(
                        Vector2::new(cx, swatch_cy - half),
                        Vector2::new(cx - half, swatch_cy + half),
                        Vector2::new(cx + half, swatch_cy + half),
                        entry.color,
                    );
                }
            }
            // Draw label text
            let text_origin = Screenpoint::new(swatch_x + 2.0 * configs.indicator_gap, row_y);
            let label = TextLabel::new(&entry.label, text_origin);
            label.plot(rl, &configs.label_style);
        }
    }

    fn data_bounds(&self) -> super::view::DataBBox {
        unimplemented!("Doesn't make sense for legend")
    }
}

impl Themable for LegendConfig {
    fn apply_theme(&mut self, scheme: &crate::colorscheme::Colorscheme) {
        self.label_style.apply_theme(scheme);
    }
}
