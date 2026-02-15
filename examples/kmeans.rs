#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]

mod common;
use common::{KMeans, KMeansPlotBuilder, MakeCirclesBuilder, make_circles};
use locus::{
    HEIGHT, WIDTH,
    colorscheme::GITHUB_DARK,
    graph::{Graph, GraphBuilder},
    plottable::{
        line::{Axis, GridLines, Orientation, Separation},
        view::Viewport,
    },
    plotter::PlotElement,
};
use raylib::prelude::*;
use std::f32;

const MAX_COLOR: Color = Color::RED;
const MIN_COLOR: Color = Color::BLUE;

#[allow(clippy::cast_precision_loss)]
fn main() {
    let (mut rl, rl_thread) = raylib::init()
        .width(WIDTH)
        .height(HEIGHT)
        .title("2D K Means")
        .build();
    let dataset = make_circles(
        &MakeCirclesBuilder::default()
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
    let colorscheme = GITHUB_DARK.clone();
    let graph = Graph::new(kmeans_plot);
    let graph_config: locus::graph::GraphConfig<commoncrate::::KMeansPlot<'_>> = GraphBuilder::default()
        .viewport(Viewport::new(0.0, 0.0, WIDTH as f32, HEIGHT as f32))
        .grid(grid_lines)
        .axis(axis)
        .subject_configs(KMeansPlotBuilder::default().build().unwrap())
        .colorscheme(colorscheme.clone())
        .build()
        .unwrap();
    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&rl_thread);
        d.clear_background(colorscheme.background);
        graph.plot(&mut d, &graph_config);
    }
}
