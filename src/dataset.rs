use crate::plottable::point::Datapoint;
use raylib::prelude::Vector2;

#[derive(Debug, Clone)]
pub struct Dataset {
    pub data: Vec<Datapoint>,
    pub range_max: Vector2,
    pub range_min: Vector2,
}

impl Dataset {
    #[must_use]
    pub fn new(data: Vec<impl Into<Datapoint>>) -> Self {
        let data: Vec<Datapoint> = data
            .into_iter()
            .map(std::convert::Into::into)
            .collect::<Vec<_>>();
        if data.is_empty() {
            return Self {
                data,
                range_max: Vector2::zero(),
                range_min: Vector2::zero(),
            };
        }

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
}
