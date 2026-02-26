# Locus

Locus is a composable graphing and plotting library for Rust, built on top of
[raylib](https://crates.io/crates/raylib). It provides a trait-driven
architecture for visualizing 2D data in real-time windows, with automatic axis
fitting, tick generation, color theming, and per-point attribute mapping.

![Main View Placeholder](docs/images/main_view.png)

## Features

* **Trait-driven rendering.** Two core traits (`ChartElement` for data-space
  elements, `PlotElement` for screen-space elements) make it easy to add custom
  visual primitives.
* **Builder-driven configuration.** Every visual element exposes a builder so
  that complex graphs remain readable and composable.
* **Automatic "nice number" axes.** Axis ranges and tick positions snap to
  multiples of 1, 2, or 5 for clean, human-friendly labels.
* **Multiple scale types.** Linear, logarithmic, and symmetric-log tick
  generation is built in.
* **Built-in color schemes.** Dracula, Nord, Viridis, Solarized (dark/light),
  GitHub (dark/light), and Matplotlib palettes are ready to use, and custom
  schemes are trivially constructed.
* **Per-point dynamic attributes.** Scatter plot size, color, and shape can be
  fixed or driven by a closure over each data point.
* **Rich chrome.** Titles, axis labels, tick labels, grid lines, legends (with
  shape indicators), and data-space annotations with leader arrows.

## Quick start

Add Locus to your `Cargo.toml`:

```toml
[dependencies]
locus = { git = "https://github.com/Marco-Normal/Locus" }
raylib = "5.5.1"
```

Then create a minimal scatter plot:

```rust
use locus::{
    HEIGHT, WIDTH,
    colorscheme::GITHUB_DARK,
    dataset::Dataset,
    graph::{ConfiguredElement, Graph, GraphBuilder},
    plottable::{
        line::{Axis, GridLines, Orientation, TickLabels},
        scatter::{ScatterPlot, ScatterPlotBuilder},
        view::{Margins, Viewport},
    },
    plotter::PlotElement,
};
use raylib::prelude::*;

fn main() {
    let (mut rl, thread) = raylib::init()
        .width(WIDTH)
        .height(HEIGHT)
        .title("Quick Start")
        .build();

    let data = Dataset::new(vec![
        (1.0, 2.0), (3.0, 7.0), (5.0, 3.0),
        (7.0, 8.0), (9.0, 5.0),
    ]);
    let scatter = ScatterPlot::new(&data);
    let graph = Graph::new(scatter);
    let scheme = GITHUB_DARK.clone();
    let axis = Axis::fitting(
        data.range_min.x..data.range_max.x,
        data.range_min.y..data.range_max.y,
        0.05,
        10,
    );

    let config = GraphBuilder::default()
        .viewport(
            Viewport::new(0.0, 0.0, WIDTH as f32, HEIGHT as f32)
                .with_margins(Margins {
                    left: 60.0, right: 20.0, top: 50.0, bottom: 55.0,
                }),
        )
        .colorscheme(scheme.clone())
        .axis(ConfiguredElement::with_defaults(axis))
        .grid(ConfiguredElement::with_defaults(
            GridLines::new(axis, Orientation::default()),
        ))
        .ticks(ConfiguredElement::with_defaults(TickLabels::new(axis)))
        .title("Quick Start")
        .xlabel("X")
        .ylabel("Y")
        .subject_configs(
            ScatterPlotBuilder::default()
                .fixed_color(Color::RED)
                .fixed_size(5.0)
                .build()
                .unwrap(),
        )
        .build()
        .unwrap();

    while !rl.window_should_close() {
        let mut d = rl.begin_drawing(&thread);
        d.clear_background(scheme.background);
        graph.plot(&mut d, &config);
    }
}
```

## Architecture

The library is built around a strong separation between data representation and
screen rendering.

### Core traits

| Trait          | Space  | Purpose                                                                                                   |
|----------------|--------|-----------------------------------------------------------------------------------------------------------|
| `ChartElement` | Data   | Elements that live in arbitrary coordinates and need a `ViewTransformer` to be projected onto the screen. |
| `PlotElement`  | Screen | Elements drawn directly in pixel coordinates (including the fully assembled `Graph` itself).              |

### The Graph orchestrator

`Graph<T>` wraps any `ChartElement` subject and manages the surrounding chrome:

```text
Graph<T>  (implements PlotElement)
  |
  +-- Viewport + Margins
  +-- Colorscheme
  +-- Axis (optional)
  +-- GridLines (optional)
  +-- TickLabels (optional)
  +-- Title / XLabel / YLabel (optional)
  +-- Legend (optional)
  +-- Annotations (optional)
  |
  +-- Subject T  (implements ChartElement)
        |
        +-- data_bounds()  -> DataBBox
        +-- draw_in_view() using ViewTransformer
```

At render time, `Graph::plot()` constructs a `ViewTransformer` from the
subject's data bounds (or the explicit axis range) and the inner viewport, then
draws each layer in order: grid, data, axes, ticks, labels, legend, and
annotations.

### View transformation

`ViewTransformer` linearly maps data coordinates to screen pixels. The y-axis is
inverted automatically (data-y up, screen-y down) so that plots follow the
standard mathematical orientation.

## Color schemes

Locus ships with several ready-made palettes:

| Name               | Style                              |
|--------------------|------------------------------------|
| `DRACULA`          | Dark, high-contrast                |
| `NORD`             | Dark, muted Arctic                 |
| `VIRIDIS`          | Dark, perceptually uniform ramp    |
| `SOLARIZED_DARK`   | Solarized dark variant             |
| `SOLARIZED_LIGHT`  | Solarized light variant            |
| `GITHUB_DARK`      | GitHub dark mode                   |
| `GITHUB_LIGHT`     | GitHub light mode                  |
| `MATPLOTLIB_LIGHT` | Classic Matplotlib tab10 (default) |

Custom schemes are created with `Colorscheme::new(...)`, and existing schemes
can be extended with additional accent colors via `Colorscheme::extend`.

## Configuration

All visual elements use the builder pattern (via `derive_builder`). Common
patterns include:

```rust
// Customise axis appearance
ConfiguredElement::with_defaults(axis).configure(|a: &mut AxisConfigs| {
    a.x_arrow = Visibility::Invisible;
});

// Per-point dynamic coloring on a scatter plot
ScatterPlotBuilder::default()
    .mapped_color(Box::new(|pt, _i| {
        if pt.y > 0.0 { Color::GREEN } else { Color::RED }
    }))
    .build()
    .unwrap();
```

## Examples

The `examples/` directory contains runnable demonstrations:

| Example         | Description                                                            |
|-----------------|------------------------------------------------------------------------|
| `dispersion`    | Multiple scatter plots with different datasets side by side            |
| `kmeans`        | K-Means clustering visualization                                       |
| `text_showcase` | Full-featured demo: title, axis labels, ticks, legend, and annotations |

Run an example with:

```sh
cargo run --example text_showcase
```

## Gallery

![Scatter Plot Example](docs/images/scatter.png)
_Scatter plot visualization_

![K-Means Example](docs/images/kmeans.png)
_K-Means clustering visualization_

## License

This project is licensed under the GPL-3.0 license. See the repository for
details.
