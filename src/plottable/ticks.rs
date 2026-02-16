#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
#![forbid(unsafe_code)]
use std::cmp::Ordering;

use crate::plottable::{
    common::{linear_spacing, log_spacing},
    line::Separation,
};

#[derive(Debug, Clone)]
pub struct Tick {
    pub value: f32,    // data units
    pub label: String, // preformatted
    pub major: bool,   // major/minor support for log/symlog
}

#[derive(Debug, Clone, Copy, Default)]
pub enum Scale {
    #[default]
    Linear,
    Log {
        base: f32,
        include_minor: bool,
    },
    SymLog {
        base: f32,
        lin_threshold: f32, // linear region around zero, e.g. 1.0
        include_minor: bool,
    },
}

#[derive(Debug, Clone, Copy)]
pub struct TickSpec {
    pub scale: Scale,
    pub max_ticks: usize,
    pub separation: Separation, // used by Linear only
}

#[derive(Debug, Clone)]
pub struct TickSet {
    pub step: Option<f32>, // Some for Linear
    pub ticks: Vec<Tick>,
}

impl TickSet {
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
        // let mut lo = min.min(max);
        // let mut hi = min.max(max);
        // if (hi - lo).abs() < f32::EPSILON {
        //     lo -= 1.0;
        //     hi += 1.0;
        // }

        // let n = spec.max_ticks.max(2) as f32;
        // let step = match spec.separation {
        //     Separation::Value(v) if v > 0.0 && v.is_finite() => v,
        //     _ => nice_number(((hi - lo) / (n - 1.0)).max(f32::EPSILON), true),
        // };

        // let tmin = (lo / step).floor() * step;
        // let tmax = (hi / step).ceil() * step;
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
    // Keep labels compact; you can switch to scientific notation if preferred.
    if (0.01..1000.0).contains(&v) {
        format_tick(v, 6)
    } else {
        format!("{v:.0e}")
    }
}
