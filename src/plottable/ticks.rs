//! Tick generation for linear, logarithmic, and symmetric-log scales.
//!
//! This module implements the algorithms that produce "nice" tick positions
//! and their formatted labels given a data range and a [`Scale`] type.
//! The entry point is [`TickSet::generate_ticks`], which dispatches to
//! specialised routines for each scale:
//!
//! * **`Linear`** : uses the "nice number" algorithm to snap step sizes to
//!   multiples of 1, 2, or 5, producing familiar round numbers.
//! * **`Log`** : places ticks at integer powers of the chosen base, with
//!   optional minor ticks at integer multiples within each decade.
//! * **`SymLog`** : combines a linear region around zero with log wings in
//!   both the positive and negative directions, useful for data that
//!   spans several orders of magnitude while including zero.

use std::cmp::Ordering;

use crate::plottable::{
    common::{linear_spacing, log_spacing},
    line::Separation,
};

/// A single tick mark along an axis.
#[derive(Debug, Clone)]
pub struct Tick {
    /// Position of the tick in data units.
    pub value: f32,
    /// Pre-formatted label string (empty for unlabelled minor ticks).
    pub label: String,
    /// `true` for major ticks, `false` for minor ticks (log/symlog scales).
    pub major: bool,
}

/// The type of scale used to generate tick positions.
#[derive(Debug, Clone, Copy, Default)]
pub enum Scale {
    /// Uniform spacing between ticks (the default).
    #[default]
    Linear,
    /// Logarithmic spacing where ticks are placed at integer powers of
    /// `base`. When `include_minor` is `true`, additional ticks are placed
    /// at integer multiples within each decade.
    Log {
        /// Logarithm base (must be > 1).
        base: f32,
        /// Whether to include minor ticks between major ones.
        include_minor: bool,
    },
    /// Symmetric logarithmic scale: linear around zero within
    /// `lin_threshold`, logarithmic outside.
    SymLog {
        /// Logarithm base (must be > 1).
        base: f32,
        /// Half-width of the linear region centred on zero.
        lin_threshold: f32,
        /// Whether to include minor ticks in the log wings.
        include_minor: bool,
    },
}

/// Parameters that fully describe how to generate ticks for one axis.
#[derive(Debug, Clone, Copy)]
pub struct TickSpec {
    /// The scale type (linear, log, or symlog).
    pub scale: Scale,
    /// Maximum number of ticks to aim for.
    pub max_ticks: usize,
    /// Spacing strategy (used by the linear scale only).
    pub separation: Separation,
}

/// The output of a tick generation pass: an optional step size and the
/// ordered list of [`Tick`]s.
#[derive(Debug, Clone)]
pub struct TickSet {
    /// Computed step size (present for linear scales, `None` for log/symlog).
    pub step: Option<f32>,
    /// Ordered sequence of tick marks.
    pub ticks: Vec<Tick>,
}

impl TickSet {
    /// Generate a set of ticks spanning `[min, max]` according to `spec`.
    ///
    /// Dispatches to the appropriate algorithm based on [`TickSpec::scale`].
    #[must_use]
    pub fn generate_ticks(min: f32, max: f32, spec: TickSpec) -> Self {
        match spec.scale {
            Scale::Linear => Self::linear_ticks(min, max, spec),
            Scale::Log {
                base,
                include_minor,
            } => Self::log_ticks(min, max, base, include_minor),
            Scale::SymLog {
                base,
                lin_threshold,
                include_minor,
            } => Self::symlog_ticks(min, max, base, lin_threshold, include_minor, spec.max_ticks),
        }
    }
    #[allow(
        clippy::cast_precision_loss,
        clippy::cast_possible_truncation,
        clippy::cast_sign_loss
    )]
    /// Generates Linear ticks that span `min` and `max`, with ticks positioned at "nice" numbers
    fn linear_ticks(min: f32, max: f32, spec: TickSpec) -> Self {
        let (val_min, val_max, step) = linear_spacing(min, max, spec.max_ticks);
        let step = match spec.separation {
            Separation::Value(v) if v > 0.0 && v.is_finite() => v,
            _ => step,
        };
        // Range from k0 to k1
        let k0 = (val_min / step).round() as i32;
        let k1 = (val_max / step).round() as i32;

        let dec = decimals_for_step(step);
        let mut ticks = Vec::with_capacity((k1 - k0 + 1).max(0) as usize);
        for k in k0..=k1 {
            let mut v = (k as f32) * step;
            if v.abs() < 1e-7 * step.max(1.0) {
                v = 0.0;
            }
            ticks.push(Tick {
                value: v,
                label: format_tick(v, dec),
                major: true,
            });
        }

        TickSet {
            step: Some(step),
            ticks,
        }
    }

    fn log_ticks(min: f32, max: f32, base: f32, include_minor: bool) -> Self {
        if let Some((_, _, major_ticks, minor_ticks)) = log_spacing(min, max, base, include_minor) {
            let mut ticks: Vec<Tick> = major_ticks
                .into_iter()
                .map(|v| Tick {
                    value: v,
                    label: format_log_label(v),
                    major: true,
                })
                .collect();
            if let Some(minor_ticks) = minor_ticks {
                ticks.extend(minor_ticks.iter().map(|v| Tick {
                    value: *v,
                    label: String::new(),
                    major: false,
                }));
            }
            ticks.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
            TickSet { step: None, ticks }
        } else {
            TickSet {
                step: None,
                ticks: Vec::new(),
            }
        }
    }

    fn symlog_ticks(
        min: f32,
        max: f32,
        base: f32,
        lin_threshold: f32,
        include_minor: bool,
        max_ticks: usize,
    ) -> Self {
        let lo = min.min(max);
        let hi = min.max(max);
        match (base.partial_cmp(&1.0), lin_threshold.partial_cmp(&1.0)) {
            (Some(Ordering::Less) | None, _) | (_, None | Some(Ordering::Less)) => TickSet {
                step: None,
                ticks: Vec::new(),
            },
            (_, _) => {
                let mut ticks = Vec::new();

                // 1) linear core around zero
                let core_lo = lo.max(-lin_threshold);
                let core_hi = hi.min(lin_threshold);
                if core_lo <= core_hi {
                    let core = Self::linear_ticks(
                        core_lo,
                        core_hi,
                        TickSpec {
                            scale: Scale::Linear,
                            max_ticks: max_ticks.clamp(3, 7),
                            separation: Separation::Auto,
                        },
                    );
                    ticks.extend(core.ticks.into_iter().map(|mut t| {
                        t.major = (t.value.abs() < f32::EPSILON)
                            || ((t.value.abs() - lin_threshold).abs() < f32::EPSILON);
                        t
                    }));
                }

                // 2) positive log wing [lin_threshold, +inf)
                if hi > lin_threshold {
                    let pos = Self::log_ticks(lin_threshold, hi, base, include_minor);
                    ticks.extend(pos.ticks);
                }

                // 3) negative log wing (-inf, -lin_threshold]
                if lo < -lin_threshold {
                    let neg = Self::log_ticks(lin_threshold, -lo, base, include_minor);
                    ticks.extend(neg.ticks.into_iter().map(|t| Tick {
                        value: -t.value,
                        label: if t.label.is_empty() {
                            String::new()
                        } else {
                            format!("-{}", t.label)
                        },
                        major: t.major,
                    }));
                }

                // dedup + sort
                ticks.sort_by(|a, b| a.value.partial_cmp(&b.value).unwrap());
                ticks.dedup_by(|a, b| (a.value - b.value).abs() < 1e-6);

                TickSet { step: None, ticks }
            }
        }
    }
}

#[allow(
    clippy::cast_sign_loss,
    clippy::cast_precision_loss,
    clippy::cast_possible_truncation
)]
fn decimals_for_step(step: f32) -> usize {
    if step <= 0.0 || !step.is_finite() {
        return 0;
    }
    if step >= 1.0 {
        return 0;
    }
    (-step.log10().floor()).max(0.0) as usize
}

fn format_tick(v: f32, decimals: usize) -> String {
    let mut s = format!("{v:.decimals$}");
    if decimals > 0 && s.contains('.') {
        while s.ends_with('0') {
            s.pop();
        }
        if s.ends_with('.') {
            s.pop();
        }
    }
    if s == "-0" { "0".to_string() } else { s }
}

fn format_log_label(v: f32) -> String {
    // Keep labels compact
    if (0.01..1000.0).contains(&v) {
        format_tick(v, 6)
    } else {
        format!("{v:.0e}")
    }
}
