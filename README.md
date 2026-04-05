# tui-spinner

[![Crates.io](https://img.shields.io/crates/v/tui-spinner.svg)](https://crates.io/crates/tui-spinner)
[![Docs.rs](https://docs.rs/tui-spinner/badge.svg)](https://docs.rs/tui-spinner)
[![CI](https://github.com/sorinirimies/tui-spinner/actions/workflows/ci.yml/badge.svg)](https://github.com/sorinirimies/tui-spinner/actions/workflows/ci.yml)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Customizable spinner widgets for [Ratatui](https://github.com/ratatui/ratatui) TUI applications.

## Preview

![Spinner Demo](examples/vhs/generated/spinner-demo.gif)

## Widgets

| Widget | Description |
|--------|-------------|
| `LinearSpinner` | Horizontal scrolling window or vertical bouncing dot — configurable axis, flow direction, and symbol style. |
| `RectSpinner` | Braille-dot arc that travels around the perimeter of a rectangle. Supports clockwise/counter-clockwise spin and filled/empty centre. |
| `CircleSpinner` | Braille-dot arc rotating around a circular ring computed with the midpoint circle algorithm. |
| `SquareSpinner` | Legacy alias for `RectSpinner` with `RectShape::Square`. Prefer `RectSpinner` for new code. |

## Quick Start

```toml
[dependencies]
tui-spinner = "0.1"
```

```rust
use ratatui::style::Color;
use tui_spinner::{
    Centre, CircleSpinner, Direction, Flow, LinearSpinner,
    LinearStyle, RectShape, RectSpinner, Spin,
};

// Horizontal ellipsis — default
let h = LinearSpinner::new(tick);

// Vertical bounce with diamond symbols, reversed
let v = LinearSpinner::new(tick)
    .direction(Direction::Vertical)
    .flow(Flow::Backwards)
    .linear_style(LinearStyle::Diamond)
    .active_color(Color::Cyan);

// Square braille arc, clockwise, filled centre
let sq = RectSpinner::new(tick)
    .shape(RectShape::Square(3))
    .spin(Spin::Clockwise)
    .centre(Centre::Filled)
    .outer_color(Color::Cyan)
    .inner_color(Color::DarkGray);

// Circular arc spinner
let circle = CircleSpinner::new(tick)
    .radius(5)
    .spin(Spin::CounterClockwise)
    .arc_color(Color::Magenta)
    .dim_color(Color::DarkGray);
```

## Running the Example

```sh
cargo run --example spinner
```

## Widget Reference

### `LinearSpinner`

Horizontal scrolling or vertical bounce animation along a straight axis.

```text
Horizontal (default):   ●●·  →  ·●●  →  ··●  →  ●··  → …
Vertical:               ●    ·    ·
                         ·    ●    ·
                         ·    ·    ●   → bounces back
```

| Builder method | Default | Description |
|---|---|---|
| `direction(Direction)` | `Horizontal` | Animation axis |
| `flow(Flow)` | `Forwards` | Animation flow direction |
| `linear_style(LinearStyle)` | `Classic` | Symbol pair for active/inactive slots |
| `total_slots(n)` | `3` | Total number of slots (row width or column height) |
| `lit_slots(n)` | `2` | Number of simultaneously lit slots (horizontal only) |
| `ticks_per_step(n)` | `3` | Ticks held per animation step (higher = slower) |
| `active_color(c)` | `White` | Colour of lit symbols |
| `inactive_color(c)` | `DarkGray` | Colour of dim symbols |
| `block(b)` | — | Optional `Block` wrapper |
| `style(s)` | — | Base style for the widget area |

#### `Direction`

| Variant | Description |
|---------|-------------|
| `Horizontal` | A window of lit symbols scrolls across a row (default) |
| `Vertical` | A single lit symbol bounces up and down a column |

#### `Flow`

| Variant | Description |
|---------|-------------|
| `Forwards` | Normal playback — left-to-right / upward-first bounce (default) |
| `Backwards` | Reversed playback — right-to-left / downward-first bounce |

#### `LinearStyle`

| Variant | Active | Inactive | Description |
|---------|--------|----------|-------------|
| `Classic` | `●` | `·` | Filled / middle dot (default) |
| `Square` | `■` | `□` | Filled / open square |
| `Diamond` | `◆` | `◇` | Filled / open diamond |
| `Bar` | `┃` | `│` | Heavy / light vertical bar |
| `Braille` | `⣿` | `⠿` | Full / sparse braille |
| `Arrow` | `▶`/`▼` | `▷`/`▽` | Direction-aware arrow |

---

### `RectSpinner`

A comet-like braille-dot arc that travels around the perimeter of a rectangle.

| Builder method | Default | Description |
|---|---|---|
| `shape(RectShape)` | `Square(2)` | Shape and size of the rectangle |
| `spin(Spin)` | `Clockwise` | Arc rotation direction |
| `centre(Centre)` | `Filled` | Whether the interior is filled or empty |
| `outer_color(c)` | `Cyan` | Colour of the rotating arc |
| `inner_color(c)` | `DarkGray` | Colour of the centre fill |
| `ticks_per_step(n)` | `1` | Ticks per arc step (higher = slower) |
| `alignment(a)` | `Left` | Horizontal alignment |
| `block(b)` | — | Optional `Block` wrapper |
| `style(s)` | — | Base style for the widget area |

#### `RectShape`

| Variant | Description |
|---------|-------------|
| `Square(n)` | Square character-cell output; `n` is the arc thickness / size parameter (2–8) |

#### `Spin`

| Variant | Description |
|---------|-------------|
| `Clockwise` | Arc travels clockwise (default) |
| `CounterClockwise` | Arc travels counter-clockwise |

#### `Centre`

| Variant | Description |
|---------|-------------|
| `Filled` | Solid interior block (default) |
| `Empty` | No interior fill — only the moving arc is visible |

---

### `CircleSpinner`

A comet-like arc rotating around a circular braille-dot ring. The perimeter is computed with the midpoint circle algorithm and sorted clockwise.

| Builder method | Default | Description |
|---|---|---|
| `radius(n)` | `4` | Circle radius in braille dots (minimum: 1) |
| `arc_len(n)` | `0` (auto: ¼ perimeter) | Number of perimeter dots in the bright arc |
| `spin(Spin)` | `Clockwise` | Arc rotation direction |
| `ticks_per_step(n)` | `1` | Ticks per arc step (higher = slower) |
| `arc_color(c)` | `White` | Colour of the rotating bright arc |
| `dim_color(c)` | `DarkGray` | Colour of the dim background ring |
| `alignment(a)` | `Left` | Horizontal alignment |
| `block(b)` | — | Optional `Block` wrapper |
| `style(s)` | — | Base style for the widget area |

Use `.char_size()` to query the exact rendered size in terminal characters `(cols, rows)`.

---

### `SquareSpinner`

> **Legacy alias** — delegates to the same braille-arc engine used by `RectSpinner` with a square shape. Prefer `RectSpinner` for new code.

| Builder method | Default | Description |
|---|---|---|
| `size(n)` | `2` | Arc thickness / square size (2–8) |
| `spin(Spin)` | `Clockwise` | Arc rotation direction |
| `centre(Centre)` | `Filled` | Interior fill mode |
| `arc_color(c)` | `White` | Colour of the rotating arc |
| `dim_color(c)` | `DarkGray` | Colour of the centre fill |
| `ticks_per_step(n)` | `1` | Ticks per arc step (higher = slower) |
| `alignment(a)` | `Left` | Horizontal alignment |
| `block(b)` | — | Optional `Block` wrapper |
| `style(s)` | — | Base style for the widget area |

Use `.char_size()` to query the exact rendered size in terminal characters `(cols, rows)`.

## Integration Pattern

All widgets are **stateless** — they only need a monotonically increasing `tick: u64` counter that you increment once per render frame. No mutable widget state is required.

```rust
use ratatui::Frame;
use ratatui::layout::Rect;
use tui_spinner::{Direction, LinearSpinner, RectSpinner, CircleSpinner};

struct App { tick: u64 }

fn update(app: &mut App) {
    app.tick = app.tick.wrapping_add(1);
}

fn draw(frame: &mut Frame, area: Rect, app: &App) {
    // Pick any spinner — they all take a tick counter
    frame.render_widget(
        LinearSpinner::new(app.tick).direction(Direction::Vertical),
        area,
    );
}
```

## Generating Demo GIFs

```sh
# Install VHS: https://github.com/charmbracelet/vhs
just vhs-all
```

## License

MIT — see [LICENSE](LICENSE).