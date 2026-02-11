use raylib::color::Color;

pub const IMAGE_SIZE: i32 = 90;
pub const WIDTH: i32 = 16 * IMAGE_SIZE;
pub const HEIGHT: i32 = 9 * IMAGE_SIZE;

const COLORS: &[Color] = &[
    Color::AQUA,
    Color::SIENNA,
    Color::FUCHSIA,
    Color::TEAL,
    Color::LIMEGREEN,
    Color::YELLOW,
    Color::PURPLE,
];
pub mod colorscheme;
pub mod dataset;
pub mod graph;
pub mod kmeans;
pub mod plottable;
pub mod plotter;
