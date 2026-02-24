#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]

use std::rc::Rc;

use derive_builder::Builder;
use raylib::{
    RaylibHandle, RaylibThread,
    color::Color,
    math::Vector2,
    prelude::{RaylibDraw, RaylibDrawHandle},
    text::{RaylibFont, WeakFont},
};

use crate::{colorscheme::Themable, plottable::point::Screenpoint, plotter::PlotElement};

#[derive(Debug, Clone, Copy)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub struct Anchor {
    pub h: HAlign,
    pub v: VAlign,
}

impl Anchor {
    pub const CENTER: Self = Self {
        h: HAlign::Center,
        v: VAlign::Middle,
    };
    pub const TOP_CENTER: Self = Self {
        h: HAlign::Center,
        v: VAlign::Top,
    };
    pub const TOP_LEFT: Self = Self {
        h: HAlign::Left,
        v: VAlign::Top,
    };
    pub const RIGHT_MIDDLE: Self = Self {
        h: HAlign::Right,
        v: VAlign::Middle,
    };
    pub const LEFT_MIDDLE: Self = Self {
        h: HAlign::Left,
        v: VAlign::Middle,
    };
    pub const CENTER_BOTTOM: Self = Self {
        h: HAlign::Center,
        v: VAlign::Bottom,
    };
}

/// Given an anchor-point and text box size (pixels),
/// return top-left draw position in screen space.
#[must_use]
pub fn anchor_text_top_left(text_specs: Vector2, anchor: Anchor, offsets: Vector2) -> Vector2 {
    let x = match anchor.h {
        HAlign::Left => 0.0,
        HAlign::Center => text_specs.x * 0.5,
        HAlign::Right => text_specs.y,
    };

    let y = match anchor.v {
        VAlign::Top => 0.0,
        VAlign::Middle => text_specs.y * 0.5,
        VAlign::Bottom => text_specs.y,
    };

    Vector2::new(x, y) + offsets
}

/// Shared, cloneable handle to a raylib font.
///
/// Wraps a `WeakFont` (non-owning) inside an `Rc` so that multiple
/// `TextStyle` instances can reference the same loaded font without
/// lifetime friction.
#[derive(Clone)]
pub struct FontHandle {
    pub(crate) font: Rc<WeakFont>,
}

impl std::fmt::Debug for FontHandle {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("FontHandle").finish_non_exhaustive()
    }
}

impl FontHandle {
    /// Load a `.ttf` / `.otf` from disk.
    ///
    /// `size` is the rasterised size in pixels — pick the largest size you
    /// intend to render at for best quality.
    #[allow(clippy::missing_errors_doc)]
    pub fn load<S: AsRef<str>>(
        rl: &mut RaylibHandle,
        thread: &RaylibThread,
        path: S,
        size: i32,
    ) -> Result<Self, String> {
        let font = rl
            .load_font_ex(thread, path.as_ref(), size, None)
            .map_err(|e| e.to_string())?;
        Ok(Self {
            font: Rc::new(font.make_weak()),
        })
    }

    /// Obtain a handle to raylib's built-in default font.
    #[must_use]
    pub fn default_font(rl: &RaylibHandle) -> Self {
        Self {
            font: Rc::new(rl.get_font_default()),
        }
    }

    /// Measure `text` rendered at `size` with `spacing`.
    #[must_use]
    pub fn measure(&self, text: &str, size: f32, spacing: f32) -> Vector2 {
        self.font.measure_text(text, size, spacing)
    }

    /// Access the inner `ffi::Font` for raw raylib calls.
    #[must_use]
    pub fn as_ffi(&self) -> &raylib::ffi::Font {
        self.font.as_ref()
    }
}

/// All visual / layout properties needed to render a piece of text.
///
/// Build with `TextStyleBuilder`:
/// ```ignore
/// let style = TextStyleBuilder::default()
///     .font_size(24.0)
///     .color(Some(Color::WHITE))
///     .anchor(Anchor::TOP_CENTER)
///     .build()
///     .unwrap();
/// ```
#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned")]
pub struct TextStyle {
    #[builder(default = "20.0")]
    pub font_size: f32,
    #[builder(default = "1.0")]
    pub alpha: f32,
    /// `None` → resolved from `Colorscheme.text` via `Themable`.
    #[builder(default = "None")]
    pub color: Option<Color>,
    #[builder(default = "1.0")]
    pub spacing: f32,
    /// If `None`, raylib's built-in default font is used.
    #[builder(default = "None")]
    pub font: Option<FontHandle>,
    #[builder(default = "Anchor::CENTER")]
    pub anchor: Anchor,
    /// Rotation in degrees (applied via `draw_text_pro`).
    #[builder(default = "0.0")]
    pub rotation: f32,
    /// Extra pixel offset applied *after* anchor resolution.
    #[builder(default = "Vector2::new(0.0, 0.0)")]
    pub offset: Vector2,
}

impl Default for TextStyle {
    fn default() -> Self {
        Self {
            font_size: 20.0,
            alpha: 1.0,
            color: None,
            spacing: 1.0,
            font: None,
            anchor: Anchor::CENTER,
            rotation: 0.0,
            offset: Vector2::new(0.0, 0.0),
        }
    }
}

impl TextStyle {
    /// Measure `text` using this style's font, size, and spacing.
    ///
    /// When no custom font is set the caller must provide a fallback via
    /// `default_font`; passing the draw-handle's default font works.
    #[must_use]
    pub fn measure_text(&self, text: &str, default_font: &WeakFont) -> Vector2 {
        match &self.font {
            Some(fh) => fh.measure(text, self.font_size, self.spacing),
            None => default_font.measure_text(text, self.font_size, self.spacing),
        }
    }
    /// Resolve the effective drawing colour (user-set or theme fallback).
    #[must_use]
    pub fn effective_color(&self) -> Color {
        self.color.unwrap_or(Color::BLACK).alpha(self.alpha)
    }
}

impl Themable for TextStyle {
    fn apply_theme(&mut self, scheme: &crate::colorscheme::Colorscheme) {
        if self.color.is_none() {
            self.color = Some(scheme.text);
        }
    }
}

/// A concrete screen-space text element: a string + its origin + its style.
///
/// Implements `PlotElement` so it can be rendered by `Graph::plot()`.
#[derive(Debug, Clone)]
pub struct TextLabel {
    pub text: String,
    pub position: Screenpoint,
}

impl TextLabel {
    #[must_use]
    pub fn new(text: impl Into<String>, position: impl Into<Screenpoint>) -> Self {
        Self {
            text: text.into(),
            position: position.into(),
        }
    }
}

impl PlotElement for TextLabel {
    type Config = TextStyle;

    fn plot(&self, rl: &mut RaylibDrawHandle, configs: &Self::Config) {
        let default_font = rl.get_font_default();
        let font: &WeakFont = match &configs.font {
            Some(fh) => &fh.font,
            None => &default_font,
        };
        let size = configs.measure_text(&self.text, &font);
        let tl = anchor_text_top_left(size, configs.anchor, configs.offset);
        let color = configs.effective_color();
        if configs.rotation.abs() < f32::EPSILON {
            // Fast path — no rotation
            rl.draw_text_ex(
                font,
                &self.text,
                tl + *self.position,
                configs.font_size,
                configs.spacing,
                color,
            );
        } else {
            // draw_text_pro rotates around `origin` (relative to `position`)
            rl.draw_text_pro(
                font,
                &self.text,
                tl + *self.position,
                Vector2::new(0.0, 0.0),
                configs.rotation,
                configs.font_size,
                configs.spacing,
                color,
            );
        }
    }
}
