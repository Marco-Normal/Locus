#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]

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
        point::{PointConfigBuilder, Screenpoint, Shape},
        text::{TextStyle, TextStyleBuilder},
    },
    plotter::{ChartElement, PlotElement},
};

/// Where to anchor the legend relative to the inner plotting area.
#[derive(Debug, Clone, Copy, Default)]
pub enum LegendPosition {
    #[default]
    TopRight,
    TopLeft,
    BottomRight,
    BottomLeft,
    /// Custom screen-space coordinates (top-left corner of the legend box).
    Custom(f32, f32),
}

/// A single row in the legend: colour swatch + label.
#[derive(Debug, Clone)]
pub struct LegendEntry {
    pub label: String,
    pub color: Color,
    pub shape: Shape,
}

impl LegendEntry {
    #[must_use]
    pub fn new(label: impl Into<String>, color: Color) -> Self {
        Self {
            label: label.into(),
            color,
            shape: Shape::Circle,
        }
    }

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

#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned")]
pub struct LegendConfig {
    #[builder(default)]
    pub position: LegendPosition,
    #[builder(default)]
    pub label_style: TextStyle,
    /// Semi-transparent background behind the legend box.
    #[builder(default = "Some(Color::new(0, 0, 0, 140))")]
    pub background: Option<Color>,
    /// Padding inside the background box.
    #[builder(default = "8.0")]
    pub padding: f32,
    /// Vertical spacing between entries.
    #[builder(default = "4.0")]
    pub entry_spacing: f32,
    /// Size of the colour swatch indicator.
    #[builder(default = "8.0")]
    pub indicator_size: f32,
    /// Gap between the swatch and the label text.
    #[builder(default = "6.0")]
    pub indicator_gap: f32,
    /// Optional border around the legend box `(color, thickness)`.
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
            let size = configs.label_style.measure_text(&entry.label, &font);
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
