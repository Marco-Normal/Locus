pub const IMAGE_SIZE: i32 = 90;
pub const WIDTH: i32 = 16 * IMAGE_SIZE;
pub const HEIGHT: i32 = 9 * IMAGE_SIZE;
pub mod colorscheme;
pub mod dataset;
pub mod graph;
pub mod plottable;
pub mod plotter;

// ── Convenience re-exports ──────────────────────────────────────────────────
pub use plottable::annotation::{Annotation, AnnotationPosition};
pub use plottable::legend::{Legend, LegendEntry, LegendPosition};
pub use plottable::text::{Anchor, FontHandle, HAlign, TextLabel, TextStyle, VAlign};
