//! Typed container for a collection of 2D data points.
//!
//! [`Dataset`] owns a `Vec<Datapoint>` and eagerly computes the axis-aligned
//! bounding box (`range_min`, `range_max`) on construction. This pre-computed
//! range is used by scatter plots, axis fitting, and the view transformer to
//! avoid repeated scans over the data.
//!
//! # Construction
//!
//! Any `Vec<T>` where `T: Into<Datapoint>` can be turned into a `Dataset`.
//! The most common source types are `(f32, f32)` tuples and `Vector2` values:
//!
//! ```rust
//! use locus::prelude::*;
//!
//! let ds = Dataset::new(vec![(0.0, 1.0), (2.0, 3.0), (4.0, 5.0)]);
//! assert_eq!(ds.data.len(), 3);
//! ```

use crate::plottable::point::Datapoint;
use raylib::prelude::Vector2;

/// An owned collection of [`Datapoint`]s together with the pre-computed
/// axis-aligned bounding box of the data.
///
/// The bounding box is stored in `range_min` (component-wise minimum) and
/// `range_max` (component-wise maximum). If the dataset is empty, both are
/// set to [`Vector2::zero`].
#[derive(Debug, Clone)]
pub struct Dataset {
    /// The raw data points.
    pub data: Vec<Datapoint>,
    /// Component-wise maximum of all points (`x` = max x, `y` = max y).
    pub range_max: Vector2,
    /// Component-wise minimum of all points (`x` = min x, `y` = min y).
    pub range_min: Vector2,
}

impl Dataset {
    /// Create a new `Dataset` from anything convertible into [`Datapoint`]s.
    ///
    /// Accepts `Vec<(f32, f32)>`, `Vec<Vector2>`, or `Vec<Datapoint>` and
    /// computes the bounding box in a single pass.
    #[must_use]
    pub fn new(data: Vec<impl Into<Datapoint>>) -> Self {
        let data: Vec<Datapoint> = data
            .into_iter()
            .map(std::convert::Into::into)
            .collect::<Vec<_>>();
        if data.is_empty() {
            return Self {
                data,
                range_max: Vector2::zero(),
                range_min: Vector2::zero(),
            };
        }

        let (min_x, max_x) = data.iter().fold((data[0].x, data[0].x), |acc, p| {
            (acc.0.min(p.x), acc.1.max(p.x))
        });
        let (min_y, max_y) = data.iter().fold((data[0].y, data[0].y), |acc, p| {
            (acc.0.min(p.y), acc.1.max(p.y))
        });
        Self {
            data,
            range_max: Vector2 { x: max_x, y: max_y },
            range_min: Vector2 { x: min_x, y: min_y },
        }
    }
}
