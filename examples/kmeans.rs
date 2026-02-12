#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]

mod commom;
use commom::{KMeans, KMeansPlotBuilder, MakeCirclesBuilder, make_circles};
use derive_builder::Builder;
use locus::{
    HEIGHT, WIDTH,
    colorscheme::{DRACULA, GITHUB_DARK},
    dataset::Dataset,
    graph::{Graph, GraphBuilder},
    plottable::{
        line::{Axis, GridLines, Orientation, Separation},
        point::{Point, PointConfigBuilder, Shape},
        scatter::ScatterPlotBuilder,
        view::BBox,
    },
    plotter::PlotElement,
};
use rand::Rng;
use raylib::prelude::*;
use std::{f32, ops::Range};

const MAX_COLOR: Color = Color::RED;
const MIN_COLOR: Color = Color::BLUE;

fn main() {
    let (mut rl, rl_thread) = raylib::init()
        .width(WIDTH)
        .height(HEIGHT)
        .title("2D K Means")
        .build();
    let dataset = make_circles(
        MakeCirclesBuilder::default()
            .with_equal_ranges(-50.0..50.0)
            .n_samples(15000)
            .n_circles(100)
            .radius(5.0..10.0)
            .build()
            .unwrap(),
    );
    let axis = Axis::fitting(
        dataset.range_min.x..dataset.range_max.x,
        dataset.range_min.y..dataset.range_max.y,
        0.01,
        15,
    );

    let grid_lines = GridLines::new(
        axis,
        Orientation::Both {
            separation_x: Separation::Auto,
            separation_y: Separation::Auto,
        },
    );
    let mut kmeans = KMeans::new(4, &dataset);
    kmeans.fit();
    let kmeans_plot = kmeans.plot();

    let graph = Graph::new(kmeans_plot);
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&rl_thread);
        graph.plot(
            &mut d,
            GraphBuilder::default()
                .bounding_box(BBox {
                    maximum: Point::new((WIDTH - 40) as f32, (HEIGHT - 40) as f32),
                    minimum: (40.0, 40.0).into(),
                })
                .grid(grid_lines)
                .axis(axis)
                .subject_configs(KMeansPlotBuilder::default().build().unwrap())
                .colorscheme(GITHUB_DARK.clone())
                .build()
                .unwrap(),
        );
    }
}
