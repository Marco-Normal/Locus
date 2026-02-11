#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
use std::{f32, ops::Range};

use derive_builder::Builder;
use rand::Rng;
use raylib::prelude::*;

use crate::{
    colorscheme::Themable,
    plottable::{
        point::{Point, PointConfigBuilder, Shape},
        view::{BBox, Offsets, ViewTransformer},
    },
    plotter::{ChartElement, PlotElement},
};

#[derive(Debug)]
pub struct Dataset {
    pub(crate) data: Vec<Point>,
    pub(crate) range_max: Vector2,
    pub(crate) range_min: Vector2,
}

#[derive(Debug, Clone, Builder)]
#[builder(pattern = "owned")]
#[builder(default)]
#[builder(name = "MakeCirclesBuilder")]
pub struct MakeCirclesConfig {
    n_circles: usize,
    n_samples: usize,
    radius: Range<f32>,
    x_range: Range<f32>,
    y_range: Range<f32>,
}

impl MakeCirclesBuilder {
    #[must_use]
    pub fn with_equal_ranges(self, range: Range<f32>) -> Self {
        Self {
            x_range: Some(range.clone()),
            y_range: Some(range),
            ..self
        }
    }
}

impl Default for MakeCirclesConfig {
    fn default() -> Self {
        Self {
            n_circles: 2,
            n_samples: 100,
            radius: 1.0..10.0,
            x_range: -10.0..10.0,
            y_range: -10.0..10.0,
        }
    }
}

impl Dataset {
    #[must_use]
    pub fn new(data: Vec<impl Into<Point>>) -> Self {
        let data: Vec<Point> = data
            .into_iter()
            .map(std::convert::Into::into)
            .collect::<Vec<_>>();
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

    #[must_use]
    pub fn make_circles(config: MakeCirclesConfig) -> Self {
        let mut rng = rand::rng();
        let mut radius: Vec<f32> = Vec::with_capacity(config.n_circles);
        let mut centers: Vec<Vector2> = Vec::with_capacity(config.n_circles);
        for _ in 0..config.n_circles {
            radius.push(rng.random_range(config.radius.clone()));
            centers.push(Vector2::new(
                rng.random_range(config.x_range.clone()),
                rng.random_range(config.y_range.clone()),
                // 0.0,
                // 0.0,
            ));
        }
        let (mut min_x, mut max_x) = (f32::INFINITY, f32::NEG_INFINITY);
        let (mut min_y, mut max_y) = (f32::INFINITY, f32::NEG_INFINITY);
        let mut data: Vec<Point> = Vec::with_capacity(config.n_samples);
        for i in 0..config.n_samples {
            let r = radius[i % config.n_circles] * f32::sqrt(rng.random::<f32>());
            let theta = rng.random::<f32>() * 2.0 * f32::consts::PI;
            let px = centers[i % config.n_circles].x + r * f32::cos(theta);

            let py = centers[i % config.n_circles].y + r * f32::sin(theta);
            if px >= max_x {
                max_x = px;
            } else if px <= min_x {
                min_x = px;
            }
            if py >= max_y {
                max_y = py;
            } else if py <= min_y {
                min_y = py;
            }
            data.push(Point { x: px, y: py });
        }
        Self {
            data,
            range_max: Vector2 { x: max_x, y: max_y },
            range_min: Vector2 { x: min_x, y: min_y },
        }
    }

    #[must_use]
    pub fn make_moons(config: MakeMoonsConfig) -> Self {
        let mut rng = rand::rng();
        let mut data: Vec<Point> = Vec::with_capacity(config.n_samples);
        let mut centers: Vec<Vector2> = Vec::with_capacity(config.n_moons);
        let mut radius: Vec<f32> = Vec::with_capacity(config.n_moons);
        let (mut min_x, mut max_x) = (f32::INFINITY, f32::NEG_INFINITY);
        let (mut min_y, mut max_y) = (f32::INFINITY, f32::NEG_INFINITY);
        for _ in 0..config.n_moons {
            centers.push(Vector2::new(
                rng.random_range(config.x_range.clone()),
                rng.random_range(config.y_range.clone()),
                // 0.0,
                // 0.0,
            ));
            radius.push(rng.random_range(config.radius.clone()));
        }
        for i in 0..config.n_samples {
            let mut x: f32;
            let mut y: f32;
            let should_flip = if i % config.n_moons == 0 { -1.0 } else { 1.0 };
            let theta = rng.random_range(0.0..f32::consts::PI);
            x = (radius[i % config.n_moons] * f32::cos(theta)) + centers[i % config.n_moons].x;
            y = (radius[i % config.n_moons] * f32::sin(theta) * should_flip)
                + should_flip * centers[i % config.n_moons].y;
            if config.noise {
                x += rng.random_range(-config.scale / 2.0..config.scale / 2.0);
                y += rng.random_range(-config.scale / 2.0..config.scale / 2.0);
            }
            data.push(Point { x, y });
            if x >= max_x {
                max_x = x;
            } else if x <= min_x {
                min_x = x;
            }
            if y >= max_y {
                max_y = y;
            } else if y <= min_y {
                min_y = y;
            }
        }
        Self {
            data,
            range_max: Vector2 { x: max_x, y: max_y },
            range_min: Vector2 { x: min_x, y: min_y },
        }
    }
    #[must_use]
    pub fn scatter_plot(&self) -> ScatterPlot<'_> {
        ScatterPlot { data: self }
    }
}

#[derive(Clone, Debug, Builder)]
#[builder(pattern = "owned", name = "MakeMoonsBuilder", default)]
pub struct MakeMoonsConfig {
    n_samples: usize,
    noise: bool,
    x_range: Range<f32>,
    y_range: Range<f32>,
    radius: Range<f32>,
    n_moons: usize,
    scale: f32,
}

impl MakeMoonsBuilder {
    #[must_use]
    pub fn with_equal_ranges(self, range: Range<f32>) -> Self {
        Self {
            x_range: Some(range.clone()),
            y_range: Some(range),
            ..self
        }
    }
}

impl Default for MakeMoonsConfig {
    fn default() -> Self {
        Self {
            n_samples: 100,
            noise: true,
            x_range: -10.0..10.0,
            y_range: -10.0..10.0,
            radius: 1.0..5.0,
            n_moons: 2,
            scale: 0.3,
        }
    }
}

pub struct ScatterPlot<'a> {
    data: &'a Dataset,
}

pub type DynamicSize = Box<dyn Fn(&Point, usize) -> f32>;
pub type DynamicColor = Box<dyn Fn(&Point, usize) -> Color>;
pub type DynamicShape = Box<dyn Fn(&Point, usize) -> Shape>;
pub type Dynamic<T> = Box<dyn Fn(&Point, usize) -> T>;
pub enum Strategy<T> {
    Fixed(T),
    Dynamic(Dynamic<T>),
}

#[derive(Builder)]
#[builder(pattern = "owned", name = "ScatterPlotBuilder")]
pub struct ScatterPlotConfig {
    #[builder(default)]
    offset: Offsets,
    #[builder(default)]
    bbox: BBox,
    #[builder(setter(into, strip_option), default = "None")]
    size: Option<Strategy<f32>>,
    #[builder(setter(into, strip_option), default = "None")]
    color: Option<Strategy<Color>>,
    #[builder(setter(into, strip_option), default = "None")]
    shape: Option<Strategy<Shape>>,
}

impl Default for ScatterPlotConfig {
    fn default() -> Self {
        ScatterPlotBuilder::default()
            .build()
            .expect("Will never fail")
    }
}

impl ScatterPlotBuilder {
    #[must_use]
    pub fn fixed_size(self, size: f32) -> Self {
        Self {
            size: Some(Some(Strategy::Fixed(size))),
            ..self
        }
    }
    #[must_use]
    pub fn fixed_color(self, color: Color) -> Self {
        Self {
            color: Some(Some(Strategy::Fixed(color))),
            ..self
        }
    }

    #[must_use]
    pub fn fixed_shape(self, shape: Shape) -> Self {
        Self {
            shape: Some(Some(Strategy::Fixed(shape))),
            ..self
        }
    }

    #[must_use]
    pub fn mapped_color(self, color_func: DynamicColor) -> Self {
        Self {
            color: Some(Some(Strategy::Dynamic(color_func))),
            ..self
        }
    }

    #[must_use]
    pub fn mapped_shape(self, shape_func: DynamicShape) -> Self {
        Self {
            shape: Some(Some(Strategy::Dynamic(shape_func))),
            ..self
        }
    }

    #[must_use]
    pub fn mapped_size(self, size_func: DynamicSize) -> Self {
        Self {
            size: Some(Some(Strategy::Dynamic(size_func))),
            ..self
        }
    }
}

// impl PlotConfig for ScatterPlotConfig {
//     fn with_bounds(&mut self, by: BBox) {
//         self.bbox = by;
//     }

//     fn with_offset(&mut self, by: Offsets) {
//         self.offset = by;
//     }
// }

// In dataset.rs

impl ChartElement for ScatterPlot<'_> {
    type Config = ScatterPlotConfig;

    fn draw_in_view(
        &self,
        rl: &mut RaylibDrawHandle,
        configs: Self::Config,
        view: &ViewTransformer,
    ) {
        self.data.data.iter().enumerate().for_each(|(i, p)| {
            let screen_point = view.to_screen(p);
            let size = match &configs.size {
                Some(strat) => match strat {
                    Strategy::Fixed(c) => *c,
                    Strategy::Dynamic(func) => func(p, i),
                },
                None => 5.0,
            };

            let shape = match &configs.shape {
                Some(strat) => match strat {
                    Strategy::Fixed(s) => *s,
                    Strategy::Dynamic(func) => func(p, i),
                },
                None => Shape::Circle,
            };
            let color = match &configs.color {
                Some(strat) => match strat {
                    Strategy::Fixed(c) => *c,
                    Strategy::Dynamic(func) => func(p, i),
                },
                None => Color::BLACK,
            };
            screen_point.plot(
                rl,
                PointConfigBuilder::default()
                    .offsets(configs.offset) // Apply final nudges if needed
                    .size(size)
                    .shape(shape)
                    .color(color)
                    .build()
                    .expect("Failed to build point config"),
            );
        });
    }

    fn data_bounds(&self) -> BBox {
        BBox {
            minimum: Point {
                x: self.data.range_min.x,
                y: self.data.range_min.y,
            },
            maximum: Point {
                x: self.data.range_max.x,
                y: self.data.range_max.y,
            },
        }
    }
}

impl Themable for ScatterPlotConfig {
    fn apply_theme(&mut self, scheme: &crate::colorscheme::Colorscheme) {
        match &self.color {
            Some(_) => (),
            None => {
                self.color = Some(Strategy::Fixed(
                    scheme.cycle.first().copied().unwrap_or(Color::BLACK),
                ));
            }
        }
    }
}
