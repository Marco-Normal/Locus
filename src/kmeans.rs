#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
use derive_builder::Builder;
use rand::prelude::*;
use raylib::math::Vector2;
use std::collections::HashMap;
const DEFAULT_MAX_ITER: usize = 1000;
const DEFAULT_MIN_MOV: f32 = 1e-4;
use crate::{
    COLORS,
    colorscheme::{Colorscheme, Themable},
    dataset::{Dataset, DynamicShape},
    plottable::{
        point::{Point, PointConfigBuilder, Shape},
        view::{BBox, Offsets},
    },
    plotter::{ChartElement, PlotElement},
};
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
    #[builder(default)]
    bbox: BBox,
    #[builder(default)]
    offset: Offsets,
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
            bbox: BBox::default(),
            offset: Offsets::default(),
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
        view: &crate::plottable::view::ViewTransformer,
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
        view: &crate::plottable::view::ViewTransformer,
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
