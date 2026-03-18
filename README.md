# tui-spinner

[![Crates.io](https://img.shields.io/crates/v/tui-spinner.svg)](https://crates.io/crates/tui-spinner)
[![Docs.rs](https://docs.rs/tui-spinner/badge.svg)](https://docs.rs/tui-spinner)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Customizable spinner widgets for [Ratatui](https://github.com/ratatui/ratatui) TUI applications.

## Spinners

| Widget | Description |
|--------|-------------|
| `ZedSpinner` | Vertical column of 3 dots; one "lit" dot bounces up and down — identical to the activity indicator in the [Zed editor](https://zed.dev) and GitHub Copilot panels. |
| `DotsSpinner` | Horizontal ellipsis-style spinner; a window of lit dots scrolls across a row of configurable length. |
| `BrailleSpinner` | Circular arc rendered with Unicode braille characters, rotating around a solid centre square. Ported from the Go braille spinner. |

## Quick Start

```toml
[dependencies]
tui-spinner = "0.1"
```

```rust
use ratatui::style::Color;
use tui_spinner::{ZedSpinner, DotsSpinner, BrailleSpinner};

// In your render function, pass a monotonically increasing tick counter:
frame.render_widget(ZedSpinner::new(tick).active_color(Color::Cyan), area);
frame.render_widget(DotsSpinner::new(tick).total_dots(5).lit_dots(2), area);
frame.render_widget(
    BrailleSpinner::new(tick).size(3).outer_color(Color::Magenta),
    area,
);
```

## Running the Example

```sh
cargo run --example spinner
```

## Widget Reference

### `ZedSpinner`

```
●   ← active (bounces 0 → 1 → 2 → 1 → 0)
·
·
```

| Builder method | Default | Description |
|---|---|---|
| `ticks_per_step(n)` | `3` | Ticks held per animation step |
| `active_color(c)` | `White` | Colour of the lit dot |
| `inactive_color(c)` | `DarkGray` | Colour of dim dots |
| `block(b)` | — | Optional `Block` wrapper |

### `DotsSpinner`

```
●●·   →   ·●●   →   ··●   →   ●··   →  …
```

| Builder method | Default | Description |
|---|---|---|
| `total_dots(n)` | `3` | Total dot slots |
| `lit_dots(n)` | `2` | How many are lit at once |
| `ticks_per_step(n)` | `3` | Ticks per step |
| `active_color(c)` | `White` | Colour of lit dots |
| `inactive_color(c)` | `DarkGray` | Colour of dim dots |

### `BrailleSpinner`

A rotating arc using Unicode braille characters (U+2800–U+28FF).

| Builder method | Default | Description |
|---|---|---|
| `size(n)` | `2` | Arc thickness (2–8) |
| `ticks_per_step(n)` | `1` | Ticks per arc step |
| `outer_color(c)` | `White` | Arc / outer ring colour |
| `inner_color(c)` | `DarkGray` | Centre square colour |
| `alignment(a)` | `Left` | Horizontal alignment |
| `block(b)` | — | Optional `Block` wrapper |

## Integration Pattern

All widgets are **stateless** — they only need a `tick: u64` counter.
Advance the counter in your event loop and pass it at render time:

```rust
struct App { tick: u64 }

fn update(app: &mut App) {
    app.tick = app.tick.wrapping_add(1);
}

fn draw(frame: &mut Frame, app: &App) {
    frame.render_widget(ZedSpinner::new(app.tick), area);
}
```

## License

MIT — see [LICENSE](LICENSE).
