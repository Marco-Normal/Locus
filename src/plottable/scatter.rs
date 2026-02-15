use crate::{
    colorscheme::Themable,
    dataset::Dataset,
    plottable::{
        point::{Point, PointConfigBuilder, Shape},
        view::{BBox, ViewTransformer},
    },
    plotter::{ChartElement, PlotElement},
};
use derive_builder::Builder;
use raylib::prelude::Color;

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

pub struct ScatterPlot<'a> {
    pub data: &'a Dataset,
}

impl<'a> ScatterPlot<'a> {
    pub fn new(data: &'a Dataset) -> Self {
        Self { data }
    }
}

impl ChartElement for ScatterPlot<'_> {
    type Config = ScatterPlotConfig;

    fn draw_in_view(
        &self,
        rl: &mut raylib::prelude::RaylibDrawHandle,
        configs: &Self::Config,
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
                &PointConfigBuilder::default()
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
