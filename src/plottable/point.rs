#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
#![forbid(unsafe_code)]

use crate::plotter::PlotElement;
use derive_builder::Builder;
use raylib::math::Vector2;
use raylib::prelude::*;

pub type Point = Vector2;

#[derive(Debug, Clone, Copy)]
pub enum Shape {
    Circle,
    Triangle,
    Rectangle,
}

#[derive(Debug, Builder)]
#[builder(pattern = "owned")]
#[builder(default)]
pub struct PointConfig {
    color: Color,
    size: f32,
    shape: Shape,
}

impl Default for PointConfig {
    fn default() -> Self {
        Self {
            color: Color::RED,
            size: 10.0,
            shape: Shape::Circle,
        }
    }
}

// impl PlotConfig for PointConfig {
//     fn with_bounds(&mut self, by: BBox) {
//         self.bbox = by;
//     }

//     fn with_offset(&mut self, by: Offsets) {
//         self.offsets = by;
//     }
// }

impl PlotElement for Point {
    type Config = PointConfig;

    fn plot(&self, rl: &mut raylib::prelude::RaylibDrawHandle, configs: Self::Config) {
        let x = self.x;
        let y = self.y;
        match configs.shape {
            Shape::Circle => {
                rl.draw_circle(x as i32, y as i32, configs.size, configs.color);
            }
            Shape::Triangle => {
                rl.draw_triangle(
                    Vector2::new(
                        x + configs.size * f32::cos(330.0_f32.to_radians()),
                        y + configs.size * f32::sin(330.0_f32.to_radians()),
                    ),
                    Vector2::new(
                        x + configs.size * f32::cos(210.0_f32.to_radians()),
                        y + configs.size * f32::sin(210.0_f32.to_radians()),
                    ),
                    Vector2::new(x, y + configs.size),
                    configs.color,
                );
            }
            Shape::Rectangle => {
                rl.draw_rectangle_v(
                    Vector2::new(x, y),
                    Vector2::new(configs.size, configs.size),
                    configs.color,
                );
            }
        }
    }
}
