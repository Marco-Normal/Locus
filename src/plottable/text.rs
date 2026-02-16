use crate::plottable::point::Screenpoint;

#[derive(Debug, Clone, Copy)]
pub enum HAlign {
    Left,
    Center,
    Right,
}

#[derive(Debug, Clone, Copy)]
pub enum VAlign {
    Top,
    Middle,
    Bottom,
}

#[derive(Debug, Clone, Copy)]
pub struct Anchor {
    pub h: HAlign,
    pub v: VAlign,
}

impl Anchor {
    pub const CENTER: Self = Self {
        h: HAlign::Center,
        v: VAlign::Middle,
    };
    pub const TOP_CENTER: Self = Self {
        h: HAlign::Center,
        v: VAlign::Top,
    };
    pub const RIGHT_MIDDLE: Self = Self {
        h: HAlign::Right,
        v: VAlign::Middle,
    };
    pub const CENTER_BOTTOM: Self = Self {
        h: HAlign::Center,
        v: VAlign::Bottom,
    };
}

/// Full anchoring algorithm:
/// Given an anchor-point and text box size (pixels),
/// return top-left draw position in screen space.
#[must_use]
pub fn anchor_text_top_left(
    origin: Screenpoint,
    text_w: f32,
    text_h: f32,
    anchor: Anchor,
    offset_x: f32,
    offset_y: f32,
) -> Screenpoint {
    let mut x = origin.x;
    let mut y = origin.y;

    match anchor.h {
        HAlign::Left => {}
        HAlign::Center => x -= text_w * 0.5,
        HAlign::Right => x -= text_w,
    }

    match anchor.v {
        VAlign::Top => {}
        VAlign::Middle => y -= text_h * 0.5,
        VAlign::Bottom => y -= text_h,
    }

    Screenpoint::new(x + offset_x, y + offset_y)
}
