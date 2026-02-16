pub mod line;
pub mod point;
pub mod scatter;
pub mod text;
pub mod ticks;
pub mod view;

pub(crate) mod common {
    use crate::plottable::line::Separation;

    pub(crate) fn get_spacing(length: f32, separation: Separation, max_ticks: usize) -> f32 {
        match separation {
            Separation::Value(v) => v,
            Separation::Auto => {
                let rough_spacing = length / (max_ticks as f32).max(1.0);
                nice_number(rough_spacing, true)
            }
        }
    }

    pub(crate) fn nice_number(value: f32, round: bool) -> f32 {
        let exponent = value.log10().floor();
        let fraction = value / 10.0_f32.powf(exponent);

        let nice_fraction = if round {
            if fraction < 1.5 {
                1.0
            } else if fraction < 3.0 {
                2.0
            } else if fraction < 7.0 {
                5.0
            } else {
                10.0
            }
        } else if fraction <= 1.0 {
            1.0
        } else if fraction <= 2.0 {
            2.0
        } else if fraction <= 5.0 {
            5.0
        } else {
            10.0
        };
        nice_fraction * 10.0_f32.powf(exponent)
    }
    /// Returns a tuple where the first value is the minimum, the second maximum and the third
    /// the step size. Iterating from min to max by step, generates a "nicely" spaced range
    pub(crate) fn linear_spacing(min: f32, max: f32, max_ticks: usize) -> (f32, f32, f32) {
        let mut low = min.min(max);
        let mut high = min.max(max);
        if (high - low).abs() < f32::EPSILON {
            low -= 1.0;
            high += 1.0;
        }

        let n = max_ticks.max(2) as f32; // Guarantees at least n = 2
        let step = nice_number(((high - low) / (n - 1.0)).max(f32::EPSILON), true);
        let val_min = (low / step).floor() * step;
        let val_max = (max / step).ceil() * step;
        (val_min, val_max, step)
    }
    /// Returns a tuple composed of (min_val, max_val, ticks, minor_ticks)
    pub(crate) fn log_spacing(
        min: f32,
        max: f32,
        base: f32,
        include_minor: bool,
    ) -> Option<(f32, f32, Vec<f32>, Option<Vec<f32>>)> {
        let low = min.min(max);
        let high = min.max(max);
        match (base.partial_cmp(&1.0), high.partial_cmp(&0.0)) {
            (Some(std::cmp::Ordering::Less) | None, _)
            | (_, None | Some(std::cmp::Ordering::Less)) => None,
            _ => {
                // Log scale requires positive domain
                // Force min to be positive
                let low_pos = low.max(f32::MIN_POSITIVE);
                let e0 = low_pos.log(base).floor() as i32;
                let e1 = high.log(base).ceil() as i32;
                let mut ticks = Vec::new();
                let mut minor_ticks = if include_minor {
                    Some(Vec::new())
                } else {
                    None
                };
                {
                    for exponent in e0..=e1 {
                        let tick = base.powi(exponent);
                        if (low..high).contains(&tick) {
                            ticks.push(tick);
                        }

                        if include_minor {
                            // For base-10, minor = 2..9 * 10^e. For other bases, use integer multiples < base.
                            let minor_max = base.floor() as i32;
                            if minor_max >= 3 {
                                for m in 2..minor_max {
                                    let minor_val = (m as f32) * base.powi(exponent);
                                    if (low..high).contains(&minor_val) {
                                        if let Some(ref mut minor_ticks) = minor_ticks {
                                            minor_ticks.push(minor_val);
                                        };
                                    }
                                }
                            }
                        }
                    }
                }
                Some((low_pos, high, ticks, minor_ticks))
            }
        }
    }
}
