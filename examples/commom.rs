#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
use derive_builder::Builder;
use rand::prelude::*;
use raylib::math::Vector2;
use std::collections::HashMap;
use std::ops::Range;
const DEFAULT_MAX_ITER: usize = 1000;
const DEFAULT_MIN_MOV: f32 = 1e-4;
use locus::{
    colorscheme::{Colorscheme, Themable},
    dataset::Dataset,
    plottable::{
        point::{Point, PointConfigBuilder, Shape},
        scatter::DynamicShape,
        view::BBox,
    },
    plotter::{ChartElement, PlotElement},
};
const COLORS: &[raylib::prelude::Color] = &[
    raylib::prelude::Color::AQUA,
    raylib::prelude::Color::SIENNA,
    raylib::prelude::Color::FUCHSIA,
    raylib::prelude::Color::TEAL,
    raylib::prelude::Color::LIMEGREEN,
    raylib::prelude::Color::YELLOW,
    raylib::prelude::Color::PURPLE,
];
#[derive(Debug)]
struct Centroid {
    center: Point,
    friends: Vec<usize>,
}
#[derive(Debug)]
pub struct KMeans<'a> {
    k: usize,
    centroids: HashMap<usize, Centroid>,
    data: &'a Dataset,
    max_iter: usize,
    curr_iter: usize,
    min_mov: f32,
    has_converged: bool,
}

impl<'a> KMeans<'a> {
    #[must_use]
    pub fn new(k: usize, data: &'a Dataset) -> Self {
        let mut me = Self {
            k,
            centroids: HashMap::with_capacity(k),
            data,
            max_iter: DEFAULT_MAX_ITER,
            curr_iter: 0,
            min_mov: DEFAULT_MIN_MOV,
            has_converged: false,
        };
        me.initialize();
        me
    }
    pub fn initialize(&mut self) {
        let mut rng = rand::rng();
        for k in 0..self.k {
            let center = Point::new(
                rng.random_range(self.data.range_min.x..self.data.range_max.x),
                rng.random_range(self.data.range_min.y..self.data.range_max.y),
            );
            self.centroids.insert(
                k,
                Centroid {
                    center,
                    friends: Vec::new(),
                },
            );
        }
    }

    pub fn fit(&mut self) {
        while !self.has_converged && self.curr_iter <= self.max_iter {
            self.step();
        }
    }
    #[allow(clippy::missing_panics_doc)]
    pub fn assign(&mut self) {
        let mut mapping: HashMap<usize, Vec<usize>> = HashMap::with_capacity(self.k);
        for centroid_index in 0..self.k {
            mapping.entry(centroid_index).or_default();
        }
        for (i, p) in self.data.data.iter().enumerate() {
            let mut min_dist = f32::INFINITY;
            let mut c_index: Option<usize> = None;
            for (c, cluster) in &self.centroids {
                let distance = f32::sqrt(
                    f32::powi(cluster.center.x - p.x, 2) + f32::powi(cluster.center.y - p.y, 2),
                );
                if distance <= min_dist {
                    min_dist = distance;
                    c_index = Some(*c);
                }
            }
            assert!(c_index.is_some());
            if let Some(c_index) = c_index
                && let Some(cluster) = mapping.get_mut(&c_index)
            {
                cluster.push(i);
            }
        }
        for (c_index, friends) in mapping {
            if let Some(centroid) = self.centroids.get_mut(&c_index) {
                centroid.friends = friends;
            }
        }
    }

    pub fn update(&mut self) {
        if self.data.data.is_empty() || self.centroids.is_empty() {
            return;
        }
        let mut biggest_distance: f32 = f32::NEG_INFINITY;
        for cluster in &mut self.centroids.values_mut() {
            let points_in_cluster = cluster.friends.as_slice();
            if points_in_cluster.is_empty() {
                continue;
            }
            let mut avg_x = 0.0;
            let mut avg_y = 0.0;
            for p_index in points_in_cluster {
                let point = self.data.data[*p_index];
                avg_x += point.x;
                avg_y += point.y;
            }
            avg_x /= points_in_cluster.len() as f32;
            avg_y /= points_in_cluster.len() as f32;
            let dist = Vector2 { x: avg_x, y: avg_y }.distance_to(cluster.center);
            if dist > biggest_distance {
                biggest_distance = dist;
            }
            cluster.center.x = avg_x;
            cluster.center.y = avg_y;
        }
        if biggest_distance < self.min_mov {
            self.has_converged = true;
        }
    }
    pub fn step(&mut self) {
        if self.has_converged || self.curr_iter >= self.max_iter {
            return;
        }

        self.assign();
        self.update();
        self.curr_iter += 1;
    }
    #[must_use]
    pub fn plot(&'a self) -> KMeansPlot<'a> {
        KMeansPlot::new(self)
    }

    pub fn dynamic_plot(&'a mut self) -> DynKMeansPlot<'a> {
        DynKMeansPlot::new(self)
    }
}

#[derive(Builder)]
#[builder(pattern = "owned", name = "KMeansPlotBuilder")]
pub struct KMeansConfig {
    // #[builder(default)]
    // bbox: BBox,
    #[builder(default = "default_shape()")]
    data_shape: DynamicShape,
    #[builder(default = "default_shape()")]
    centroid_shape: DynamicShape,
    #[builder(default = "3.0")]
    data_size: f32,
    #[builder(default = "9.0")]
    centroid_size: f32,
    #[builder(default = "None", setter(into, strip_option))]
    colorscheme: Option<Colorscheme>,
}

impl Default for KMeansConfig {
    fn default() -> Self {
        Self {
            // bbox: BBox::default(),
            data_shape: default_shape(),
            centroid_shape: default_shape(),
            data_size: 3.0,
            centroid_size: 9.0,
            colorscheme: None,
        }
    }
}

fn default_shape() -> DynamicShape {
    Box::new(|_, _| Shape::Circle)
}

pub struct KMeansPlot<'a> {
    kmeans: &'a KMeans<'a>,
}

impl<'a> KMeansPlot<'a> {
    #[must_use]
    pub fn new(kmeans: &'a KMeans<'a>) -> Self {
        Self { kmeans }
    }
}

impl ChartElement for KMeansPlot<'_> {
    type Config = KMeansConfig;

    fn draw_in_view(
        &self,
        rl: &mut raylib::prelude::RaylibDrawHandle,
        configs: Self::Config,
        view: &locus::plottable::view::ViewTransformer,
    ) {
        if self.kmeans.data.data.is_empty() {
            return;
        }
        let colorscheme = configs.colorscheme.unwrap_or_default();
        for (c_index, centroid) in &self.kmeans.centroids {
            let color = colorscheme.cycle[c_index % colorscheme.cycle.len()];
            for p_index in &centroid.friends {
                let p = &self.kmeans.data.data[*p_index];
                let p = view.to_screen(p);
                p.plot(
                    rl,
                    PointConfigBuilder::default()
                        .shape((configs.data_shape)(&p, *p_index))
                        .color(color)
                        .size(configs.data_size)
                        .build()
                        .unwrap(),
                );
            }
            let centroid = view.to_screen(&centroid.center);
            centroid.plot(
                rl,
                PointConfigBuilder::default()
                    .shape((configs.centroid_shape)(&centroid, *c_index))
                    .color(color)
                    .size(configs.centroid_size)
                    .build()
                    .unwrap(),
            );
        }
    }

    fn data_bounds(&self) -> BBox {
        BBox {
            minimum: Point {
                x: self.kmeans.data.range_min.x,
                y: self.kmeans.data.range_min.y,
            },
            maximum: Point {
                x: self.kmeans.data.range_max.x,
                y: self.kmeans.data.range_max.y,
            },
        }
    }
}

impl Themable for KMeansConfig {
    fn apply_theme(&mut self, scheme: &Colorscheme) {
        if self.colorscheme.is_none() {
            self.colorscheme = Some(scheme.clone());
        }
    }
}

pub struct DynKMeansPlot<'a> {
    kmeans: &'a mut KMeans<'a>,
}

impl<'a> DynKMeansPlot<'a> {
    pub fn new(kmeans: &'a mut KMeans<'a>) -> Self {
        Self { kmeans }
    }
}

impl<'a> From<DynKMeansPlot<'a>> for KMeansPlot<'a> {
    fn from(value: DynKMeansPlot<'a>) -> Self {
        KMeansPlot {
            kmeans: value.kmeans,
        }
    }
}

impl<'a> From<&'a DynKMeansPlot<'a>> for KMeansPlot<'a> {
    fn from(value: &'a DynKMeansPlot<'a>) -> Self {
        KMeansPlot {
            kmeans: value.kmeans,
        }
    }
}

impl ChartElement for DynKMeansPlot<'_> {
    type Config = KMeansConfig;

    fn draw_in_view(
        &self,
        rl: &mut raylib::prelude::RaylibDrawHandle,
        configs: Self::Config,
        view: &locus::plottable::view::ViewTransformer,
    ) {
        if self.kmeans.data.data.is_empty() {
            return;
        }

        // if rl.is_key_pressed(KeyboardKey::KEY_N) {
        //     self.kmeans.step();
        // }
        for (c_index, centroid) in &self.kmeans.centroids {
            let color = COLORS[c_index % COLORS.len()];
            for p_index in &centroid.friends {
                let p = &self.kmeans.data.data[*p_index];
                let p = view.to_screen(p);
                p.plot(
                    rl,
                    PointConfigBuilder::default()
                        .shape((configs.data_shape)(&p, *p_index))
                        .color(color)
                        .size(configs.data_size)
                        .build()
                        .unwrap(),
                );
            }
            let centroid = view.to_screen(&centroid.center);
            centroid.plot(
                rl,
                PointConfigBuilder::default()
                    .shape((configs.centroid_shape)(&centroid, *c_index))
                    .color(color)
                    .size(configs.centroid_size)
                    .build()
                    .unwrap(),
            );
        }
    }

    fn data_bounds(&self) -> BBox {
        BBox {
            minimum: Point {
                x: self.kmeans.data.range_min.x,
                y: self.kmeans.data.range_min.y,
            },
            maximum: Point {
                x: self.kmeans.data.range_max.x,
                y: self.kmeans.data.range_max.y,
            },
        }
    }
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
            n_circles: 3,
            n_samples: 100,
            radius: 1.0..10.0,
            x_range: -10.0..10.0,
            y_range: -10.0..10.0,
        }
    }
}

pub fn make_circles(config: MakeCirclesConfig) -> Dataset {
    let mut rng = rand::rng();
    let mut radius: Vec<f32> = Vec::with_capacity(config.n_circles);
    let mut centers: Vec<Vector2> = Vec::with_capacity(config.n_circles);
    for _ in 0..config.n_circles {
        radius.push(rng.random_range(config.radius.clone()));
        centers.push(Vector2::new(
            rng.random_range(config.x_range.clone()),
            rng.random_range(config.y_range.clone()),
        ));
    }
    // let (mut min_x, mut max_x) = (f32::INFINITY, f32::NEG_INFINITY);
    // let (mut min_y, mut max_y) = (f32::INFINITY, f32::NEG_INFINITY);
    let mut data: Vec<Point> = Vec::with_capacity(config.n_samples);
    for i in 0..config.n_samples {
        let r = radius[i % config.n_circles] * f32::sqrt(rng.random::<f32>());
        let theta = rng.random::<f32>() * 2.0 * std::f32::consts::PI;
        let px = centers[i % config.n_circles].x + r * f32::cos(theta);

        let py = centers[i % config.n_circles].y + r * f32::sin(theta);
        // if px >= max_x {
        //     max_x = px;
        // } else if px <= min_x {
        //     min_x = px;
        // }
        // if py >= max_y {
        //     max_y = py;
        // } else if py <= min_y {
        //     min_y = py;
        // }
        data.push(Point { x: px, y: py });
    }
    Dataset::new(data)
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
pub fn make_moons(config: MakeMoonsConfig) -> Dataset {
    let mut rng = rand::rng();
    let mut data: Vec<Point> = Vec::with_capacity(config.n_samples);
    let mut centers: Vec<Vector2> = Vec::with_capacity(config.n_moons);
    let mut radius: Vec<f32> = Vec::with_capacity(config.n_moons);
    // let (mut min_x, mut max_x) = (f32::INFINITY, f32::NEG_INFINITY);
    // let (mut min_y, mut max_y) = (f32::INFINITY, f32::NEG_INFINITY);
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
        let should_flip = if i % 2 == 0 { -1.0 } else { 1.0 };
        let theta = rng.random_range(0.0..std::f32::consts::PI);
        x = (radius[i % config.n_moons] * f32::cos(theta)) + centers[i % config.n_moons].x;
        y = (radius[i % config.n_moons] * f32::sin(theta) * should_flip)
            + should_flip * centers[i % config.n_moons].y;
        if config.noise {
            x += rng.random_range(-config.scale / 2.0..config.scale / 2.0);
            y += rng.random_range(-config.scale / 2.0..config.scale / 2.0);
        }
        data.push(Point { x, y });
        // if x >= max_x {
        //     max_x = x;
        // } else if x <= min_x {
        //     min_x = x;
        // }
        // if y >= max_y {
        //     max_y = y;
        // } else if y <= min_y {
        //     min_y = y;
        // }
    }
    Dataset::new(data)
}
