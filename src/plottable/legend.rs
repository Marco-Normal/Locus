#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]

use derive_builder::Builder;
use raylib::{color::Color, math::Vector2, prelude::RaylibDraw};

use crate::{
    colorscheme::Themable,
    plottable::{
        point::{Screenpoint, Shape},
        text::{Anchor, TextStyle},
        view::ScreenBBox,
    },
};

// ── Legend position ──────────────────────────────────────────────────

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

// ── Legend entry ─────────────────────────────────────────────────────

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

// ── Legend ────────────────────────────────────────────────────────────

/// A drawable legend that pairs colour swatches with text labels.
///
/// Constructed via `LegendBuilder` and added to a `Graph` with
/// `.legend(entries)` or `.legend_styled(entries, |c| ...)`.
#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned")]
pub struct Legend {
    #[builder(default)]
    pub entries: Vec<LegendEntry>,
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

impl Default for Legend {
    fn default() -> Self {
        Self {
            entries: Vec::new(),
            position: LegendPosition::default(),
            label_style: TextStyle {
                font_size: 14.0,
                anchor: Anchor::TOP_LEFT,
                ..TextStyle::default()
            },
            background: Some(Color::new(0, 0, 0, 140)),
            padding: 8.0,
            entry_spacing: 4.0,
            indicator_size: 8.0,
            indicator_gap: 6.0,
            border: None,
        }
    }
}

impl Legend {
    /// Draw the legend in screen space, positioned relative to `inner_bbox`.
    #[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
    pub fn draw(
        &self,
        rl: &mut raylib::prelude::RaylibDrawHandle,
        inner_bbox: &ScreenBBox,
    ) {
        if self.entries.is_empty() {
            return;
        }

        let default_font = rl.get_font_default();

        // ── Measure all entries to compute legend box size ────────
        let row_height = self.label_style.font_size;
        let n = self.entries.len();
        let total_height = self.padding * 2.0
            + (n as f32) * row_height
            + ((n.saturating_sub(1)) as f32) * self.entry_spacing;

        let mut max_label_width: f32 = 0.0;
        for entry in &self.entries {
            let size = self.label_style.measure_text(&entry.label, &default_font);
            max_label_width = max_label_width.max(size.x);
        }
        let total_width =
            self.padding * 2.0 + self.indicator_size + self.indicator_gap + max_label_width;

        // ── Compute top-left of the legend box ───────────────────
        let (box_x, box_y) = match self.position {
            LegendPosition::TopRight => (
                inner_bbox.maximum.x - total_width - 4.0,
                inner_bbox.minimum.y + 4.0,
            ),
            LegendPosition::TopLeft => (inner_bbox.minimum.x + 4.0, inner_bbox.minimum.y + 4.0),
            LegendPosition::BottomRight => (
                inner_bbox.maximum.x - total_width - 4.0,
                inner_bbox.maximum.y - total_height - 4.0,
            ),
            LegendPosition::BottomLeft => (
                inner_bbox.minimum.x + 4.0,
                inner_bbox.maximum.y - total_height - 4.0,
            ),
            LegendPosition::Custom(x, y) => (x, y),
        };

        // ── Background ───────────────────────────────────────────
        if let Some(bg) = self.background {
            rl.draw_rectangle_v(
                Vector2::new(box_x, box_y),
                Vector2::new(total_width, total_height),
                bg,
            );
        }

        // ── Border ───────────────────────────────────────────────
        if let Some((border_color, thickness)) = self.border {
            rl.draw_rectangle_lines_ex(
                raylib::ffi::Rectangle {
                    x: box_x,
                    y: box_y,
                    width: total_width,
                    height: total_height,
                },
                thickness,
                border_color,
            );
        }

        // ── Entries ──────────────────────────────────────────────
        for (i, entry) in self.entries.iter().enumerate() {
            let row_y = box_y + self.padding + (i as f32) * (row_height + self.entry_spacing);
            let swatch_x = box_x + self.padding;
            let swatch_cy = row_y + row_height * 0.5;

            // Draw colour indicator
            match entry.shape {
                Shape::Circle => {
                    rl.draw_circle(
                        swatch_x as i32 + (self.indicator_size * 0.5) as i32,
                        swatch_cy as i32,
                        self.indicator_size * 0.5,
                        entry.color,
                    );
                }
                Shape::Rectangle => {
                    rl.draw_rectangle_v(
                        Vector2::new(swatch_x, swatch_cy - self.indicator_size * 0.5),
                        Vector2::new(self.indicator_size, self.indicator_size),
                        entry.color,
                    );
                }
                Shape::Triangle => {
                    let cx = swatch_x + self.indicator_size * 0.5;
                    let half = self.indicator_size * 0.5;
                    rl.draw_triangle(
                        Vector2::new(cx, swatch_cy - half),
                        Vector2::new(cx - half, swatch_cy + half),
                        Vector2::new(cx + half, swatch_cy + half),
                        entry.color,
                    );
                }
            }

            // Draw label text
            let text_origin = Screenpoint::new(
                swatch_x + self.indicator_size + self.indicator_gap,
                row_y,
            );
            self.label_style.draw(rl, &entry.label, text_origin);
        }
    }
}

impl Themable for Legend {
    fn apply_theme(&mut self, scheme: &crate::colorscheme::Colorscheme) {
        self.label_style.apply_theme(scheme);
    }
}
