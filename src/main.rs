#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
use knn::{
    HEIGHT, WIDTH,
    dataset::{Dataset, MakeCirclesBuilder},
    graph::{Graph, GraphBuilder},
    kmeans::{KMeans, KMeansPlotBuilder},
    plottable::{
        line::{Axis, GridLines},
        point::Point,
        view::BBox,
    },
    plotter::PlotElement,
};

fn main() {
    let (mut rl, rl_thread) = raylib::init()
        .width(WIDTH)
        .height(HEIGHT)
        .title("2D K Means")
        .build();
    let dataset = Dataset::make_circles(
        MakeCirclesBuilder::default()
            .with_equal_ranges(-50.0..50.0)
            .n_samples(15000)
            .n_circles(4)
            .radius(5.0..10.0)
            .build()
            .unwrap(),
    );
    let axis = Axis::from(&dataset);

    let grid_lines = GridLines::new(
        axis,
        knn::plottable::line::Orientation::Both {
            separation_x: knn::plottable::line::Separation::Auto,
            separation_y: knn::plottable::line::Separation::Auto,
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
                .subject_configs(
                    KMeansPlotBuilder::default()
                        .centroid_size(15.0)
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
        );
        // kmeans.plot(&mut d, KMeansConfig::default());
    }
}
