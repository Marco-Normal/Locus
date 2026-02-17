#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
use std::f32;
mod common;
use common::{make_circles, make_moons};
use locus::{
    HEIGHT, WIDTH,
    colorscheme::GITHUB_DARK,
    graph::{ConfiguredElement, Graph, GraphBuilder},
    plottable::{
        line::{Axis, TickLabels, Visibility},
        scatter::{ScatterPlot, ScatterPlotBuilder},
        ticks::Scale,
        view::Viewport,
    },
    plotter::PlotElement,
};
use raylib::prelude::*;

use common::{MakeCirclesBuilder, MakeMoonsBuilder};
#[allow(clippy::cast_possible_truncation, clippy::cast_precision_loss)]
fn main() {
    let (mut rl, rl_thread) = raylib::init()
        .width(WIDTH)
        .height(HEIGHT)
        .title("Datasets")
        .build();
    let d1 = make_circles(
        &MakeCirclesBuilder::default()
            .n_circles(10)
            .radius(5.0..10.0)
            .with_equal_ranges(-10.0..10.0)
            .n_samples(4000)
            .build()
            .unwrap(),
    );
    let d2 = make_moons(
        &MakeMoonsBuilder::default()
            .with_equal_ranges(-10.0..10.0)
            .n_moons(9)
            .noise(true)
            .scale(0.5)
            .n_samples(2000)
            .build()
            .unwrap(),
    );
    let s1 = ScatterPlot::new(&d1);
    let s2 = ScatterPlot::new(&d2);

    let colorscheme = GITHUB_DARK.clone();
    let g1 = Graph::new(s1);
    let g2 = Graph::new(s2);
    let axis = Axis::fitting(
        d1.range_min.x..d1.range_max.x,
        d1.range_min.y..d1.range_max.y,
        0.01,
        15,
    );
    let axis_d2 = Axis::fitting(
        d2.range_min.x..d2.range_max.x,
        d2.range_min.y..d2.range_max.y,
        0.01,
        15,
    );
    let ticks_1 = TickLabels::new(axis);

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&rl_thread);
        d.clear_background(colorscheme.background);
        g1.plot(
            &mut d,
            &GraphBuilder::default()
                .viewport(
                    Viewport::new(10.0, 10.0, (WIDTH / 2) as f32, (HEIGHT - 15) as f32)
                        .with_margins(locus::plottable::view::Margins {
                            left: 40.0,
                            right: 10.0,
                            top: 10.0,
                            bottom: 30.0,
                        }),
                )
                .colorscheme(colorscheme.clone())
                .axis(ConfiguredElement::with_defaults(axis).configure(|a| {
                    a.x_arrow = Visibility::Invisible;
                }))
                .ticks(ConfiguredElement::with_defaults(ticks_1).configure(|t| {
                    t.x_axis_scale = Scale::Linear;
                }))
                .subject_configs(
                    ScatterPlotBuilder::default()
                        .fixed_color(Color::RED)
                        .fixed_size(3.0)
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
        );
        g2.plot(
            &mut d,
            &GraphBuilder::default()
                .viewport(
                    Viewport::new(
                        (WIDTH / 2) as f32,
                        10.0,
                        (WIDTH / 2) as f32,
                        (HEIGHT - 15) as f32,
                    )
                    .with_margins(locus::plottable::view::Margins {
                        left: 40.0,
                        right: 10.0,
                        top: 10.0,
                        bottom: 30.0,
                    }),
                )
                .colorscheme(colorscheme.clone())
                .axis(ConfiguredElement::with_defaults(axis_d2))
                .subject_configs(
                    ScatterPlotBuilder::default()
                        .fixed_color(Color::BLUE)
                        .fixed_size(3.0)
                        .build()
                        .unwrap(),
                )
                .build()
                .unwrap(),
        );
    }
}
