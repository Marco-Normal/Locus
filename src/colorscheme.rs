#![allow(dead_code)]
#![warn(clippy::pedantic)]
#![deny(clippy::style, clippy::perf, clippy::correctness, clippy::complexity)]
use raylib::color::Color;
use std::sync::LazyLock;

pub trait Themable {
    fn apply_theme(&mut self, scheme: &Colorscheme);
}

#[derive(Clone, Debug)]
pub struct Colorscheme {
    pub background: Color,
    pub grid: Color,
    pub text: Color,
    pub axis: Color,
    pub cycle: Vec<Color>,
}

impl Colorscheme {
    #[must_use]
    pub fn new(
        background: Color,
        grid: Color,
        text: Color,
        axis: Color,
        cycle: Vec<Color>,
    ) -> Self {
        Self {
            background,
            grid,
            text,
            axis,
            cycle,
        }
    }

    pub fn extend_in_place(&mut self, other: Vec<Color>) {
        self.cycle.extend(other);
    }
    #[must_use]
    pub fn extend(self, other: Vec<Color>) -> Self {
        let mut cycle = self.cycle.clone();
        cycle.extend(other);
        Self { cycle, ..self }
    }
}

impl Default for Colorscheme {
    fn default() -> Self {
        MATPLOTLIB_LIGHT.clone()
    }
}

pub static DRACULA: LazyLock<Colorscheme> = LazyLock::new(|| Colorscheme {
    background: Color {
        r: 40,
        g: 42,
        b: 54,
        a: 255,
    },
    text: Color {
        r: 248,
        g: 248,
        b: 242,
        a: 255,
    },
    grid: Color {
        r: 68,
        g: 71,
        b: 90,
        a: 200,
    },

    axis: Color {
        r: 68,
        g: 71,
        b: 90,
        a: 255,
    },
    cycle: vec![
        Color {
            r: 255,
            g: 85,
            b: 85,
            a: 255,
        }, // Red
        Color {
            r: 255,
            g: 184,
            b: 108,
            a: 255,
        }, // Orange
        Color {
            r: 241,
            g: 150,
            b: 140,
            a: 255,
        }, // Yellow
        Color {
            r: 80,
            g: 250,
            b: 123,
            a: 255,
        }, // Green
        Color {
            r: 139,
            g: 233,
            b: 253,
            a: 255,
        }, // Cyan
        Color {
            r: 189,
            g: 147,
            b: 249,
            a: 255,
        }, // Purple
        Color {
            r: 255,
            g: 121,
            b: 198,
            a: 255,
        }, // Pink
    ],
});
pub static NORD: LazyLock<Colorscheme> = LazyLock::new(|| Colorscheme {
    background: Color {
        r: 46,
        g: 52,
        b: 64,
        a: 255,
    },
    text: Color {
        r: 216,
        g: 222,
        b: 233,
        a: 255,
    },
    grid: Color {
        r: 76,
        g: 86,
        b: 106,
        a: 150,
    }, // Nord3 (Polar Night)
    axis: Color {
        r: 67,
        g: 76,
        b: 94,
        a: 255,
    }, // Nord2
    cycle: vec![
        Color {
            r: 191,
            g: 97,
            b: 106,
            a: 255,
        }, // Red
        Color {
            r: 208,
            g: 135,
            b: 112,
            a: 255,
        }, // Orange
        Color {
            r: 235,
            g: 203,
            b: 139,
            a: 255,
        }, // Yellow
        Color {
            r: 163,
            g: 190,
            b: 140,
            a: 255,
        }, // Green
        Color {
            r: 136,
            g: 192,
            b: 208,
            a: 255,
        }, // Cyan
        Color {
            r: 129,
            g: 161,
            b: 193,
            a: 255,
        }, // Blue
        Color {
            r: 180,
            g: 142,
            b: 173,
            a: 255,
        }, // Purple
    ],
});
pub static VIRIDIS: LazyLock<Colorscheme> = LazyLock::new(|| Colorscheme {
    background: Color {
        r: 34,
        g: 34,
        b: 34,
        a: 255,
    },
    text: Color {
        r: 240,
        g: 240,
        b: 240,
        a: 255,
    },
    grid: Color {
        r: 240,
        g: 240,
        b: 240,
        a: 40,
    }, // Subtle light grid
    axis: Color {
        r: 80,
        g: 80,
        b: 80,
        a: 255,
    }, // Solid gray axis
    cycle: vec![
        Color {
            r: 68,
            g: 1,
            b: 84,
            a: 255,
        }, // Purple
        Color {
            r: 59,
            g: 82,
            b: 139,
            a: 255,
        }, // Blue
        Color {
            r: 33,
            g: 145,
            b: 140,
            a: 255,
        }, // Teal
        Color {
            r: 94,
            g: 201,
            b: 98,
            a: 255,
        }, // Green
        Color {
            r: 253,
            g: 231,
            b: 37,
            a: 255,
        }, // Yellow
    ],
});

pub static SOLARIZED_DARK: LazyLock<Colorscheme> = LazyLock::new(|| Colorscheme {
    background: Color {
        r: 0,
        g: 43,
        b: 54,
        a: 255,
    }, // Base03
    text: Color {
        r: 131,
        g: 148,
        b: 150,
        a: 255,
    }, // Base0
    grid: Color {
        r: 7,
        g: 54,
        b: 66,
        a: 200,
    }, // Base02
    axis: Color {
        r: 88,
        g: 110,
        b: 117,
        a: 255,
    }, // Base01
    cycle: vec![
        Color {
            r: 181,
            g: 137,
            b: 0,
            a: 255,
        }, // Yellow
        Color {
            r: 203,
            g: 75,
            b: 22,
            a: 255,
        }, // Orange
        Color {
            r: 220,
            g: 50,
            b: 47,
            a: 255,
        }, // Red
        Color {
            r: 211,
            g: 54,
            b: 130,
            a: 255,
        }, // Magenta
        Color {
            r: 108,
            g: 113,
            b: 196,
            a: 255,
        }, // Violet
        Color {
            r: 38,
            g: 139,
            b: 210,
            a: 255,
        }, // Blue
        Color {
            r: 42,
            g: 161,
            b: 152,
            a: 255,
        }, // Cyan
        Color {
            r: 133,
            g: 153,
            b: 0,
            a: 255,
        }, // Green
    ],
});

pub static GITHUB_DARK: LazyLock<Colorscheme> = LazyLock::new(|| Colorscheme {
    background: Color {
        r: 13,
        g: 17,
        b: 23,
        a: 255,
    },
    text: Color {
        r: 201,
        g: 209,
        b: 217,
        a: 255,
    },
    grid: Color {
        r: 48,
        g: 54,
        b: 61,
        a: 180,
    },
    axis: Color {
        r: 48,
        g: 54,
        b: 61,
        a: 255,
    },
    cycle: vec![
        Color {
            r: 126,
            g: 231,
            b: 135,
            a: 255,
        }, // Green
        Color {
            r: 121,
            g: 192,
            b: 255,
            a: 255,
        }, // Blue
        Color {
            r: 210,
            g: 153,
            b: 255,
            a: 255,
        }, // Purple
        Color {
            r: 255,
            g: 123,
            b: 114,
            a: 255,
        }, // Red
        Color {
            r: 255,
            g: 166,
            b: 87,
            a: 255,
        }, // Orange
        Color {
            r: 210,
            g: 178,
            b: 132,
            a: 255,
        }, // Tan
    ],
});

pub static MATPLOTLIB_LIGHT: LazyLock<Colorscheme> = LazyLock::new(|| Colorscheme {
    background: Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    },
    text: Color {
        r: 30,
        g: 30,
        b: 30,
        a: 255,
    },
    grid: Color {
        r: 150,
        g: 150,
        b: 150,
        a: 255,
    }, // Standard light gray grid
    axis: Color {
        r: 0,
        g: 0,
        b: 0,
        a: 255,
    }, // Solid black axis
    cycle: vec![
        Color {
            r: 31,
            g: 119,
            b: 180,
            a: 255,
        }, // Blue
        Color {
            r: 255,
            g: 127,
            b: 14,
            a: 255,
        }, // Orange
        Color {
            r: 44,
            g: 160,
            b: 44,
            a: 255,
        }, // Green
        Color {
            r: 214,
            g: 39,
            b: 40,
            a: 255,
        }, // Red
        Color {
            r: 148,
            g: 103,
            b: 189,
            a: 255,
        }, // Purple
        Color {
            r: 140,
            g: 86,
            b: 75,
            a: 255,
        }, // Brown
        Color {
            r: 227,
            g: 119,
            b: 194,
            a: 255,
        }, // Pink
    ],
});

pub static SOLARIZED_LIGHT: LazyLock<Colorscheme> = LazyLock::new(|| Colorscheme {
    background: Color {
        r: 253,
        g: 246,
        b: 227,
        a: 255,
    },
    text: Color {
        r: 101,
        g: 123,
        b: 131,
        a: 255,
    }, // Base00
    grid: Color {
        r: 238,
        g: 232,
        b: 213,
        a: 255,
    }, // Base2
    axis: Color {
        r: 147,
        g: 161,
        b: 161,
        a: 255,
    }, // Base1
    cycle: vec![
        Color {
            r: 181,
            g: 137,
            b: 0,
            a: 255,
        },
        Color {
            r: 203,
            g: 75,
            b: 22,
            a: 255,
        },
        Color {
            r: 220,
            g: 50,
            b: 47,
            a: 255,
        },
        Color {
            r: 211,
            g: 54,
            b: 130,
            a: 255,
        },
        Color {
            r: 38,
            g: 139,
            b: 210,
            a: 255,
        },
        Color {
            r: 133,
            g: 153,
            b: 0,
            a: 255,
        },
    ],
});

pub static GITHUB_LIGHT: LazyLock<Colorscheme> = LazyLock::new(|| Colorscheme {
    background: Color {
        r: 255,
        g: 255,
        b: 255,
        a: 255,
    },
    text: Color {
        r: 31,
        g: 35,
        b: 40,
        a: 255,
    },
    grid: Color {
        r: 208,
        g: 215,
        b: 222,
        a: 150,
    }, // Muted blue-gray
    axis: Color {
        r: 31,
        g: 35,
        b: 40,
        a: 255,
    },
    cycle: vec![
        Color {
            r: 5,
            g: 152,
            b: 250,
            a: 255,
        },
        Color {
            r: 26,
            g: 127,
            b: 55,
            a: 255,
        },
        Color {
            r: 207,
            g: 34,
            b: 46,
            a: 255,
        },
        Color {
            r: 154,
            g: 103,
            b: 0,
            a: 255,
        },
        Color {
            r: 130,
            g: 80,
            b: 223,
            a: 255,
        },
    ],
});
