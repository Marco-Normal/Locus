#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
#![forbid(unsafe_code)]

use crate::plotter::PlotElement;
use derive_builder::Builder;
use raylib::math::Vector2;
use raylib::prelude::*;

#[derive(Clone, Copy, Debug)]
pub struct Datapoint(pub Vector2);

impl Datapoint {
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self((x, y).into())
    }
}

impl From<Vector2> for Datapoint {
    fn from(value: Vector2) -> Self {
        Self(value)
    }
}

impl From<&Vector2> for Datapoint {
    fn from(value: &Vector2) -> Self {
        Self(*value)
    }
}

impl From<(f32, f32)> for Datapoint {
    fn from(value: (f32, f32)) -> Self {
        Datapoint(value.into())
    }
}

impl std::ops::DerefMut for Datapoint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for Datapoint {
    type Target = Vector2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}
#[derive(Clone, Copy, Debug)]
pub struct Screenpoint(pub Vector2);

impl Screenpoint {
    #[must_use]
    pub fn new(x: f32, y: f32) -> Self {
        Self((x, y).into())
    }
}
impl From<Vector2> for Screenpoint {
    fn from(value: Vector2) -> Self {
        Self(value)
    }
}

impl From<&Vector2> for Screenpoint {
    fn from(value: &Vector2) -> Self {
        Self(*value)
    }
}

impl From<(f32, f32)> for Screenpoint {
    fn from(value: (f32, f32)) -> Self {
        Screenpoint(value.into())
    }
}

impl std::ops::DerefMut for Screenpoint {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

impl std::ops::Deref for Screenpoint {
    type Target = Vector2;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

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

impl PlotElement for Screenpoint {
    type Config = PointConfig;
    #[allow(clippy::cast_possible_truncation)]
    fn plot(&self, rl: &mut raylib::prelude::RaylibDrawHandle, configs: &PointConfig) {
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
