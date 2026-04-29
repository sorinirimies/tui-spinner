# tui-spinner

[![Crates.io](https://img.shields.io/crates/v/tui-spinner.svg)](https://crates.io/crates/tui-spinner)
[![Docs.rs](https://docs.rs/tui-spinner/badge.svg)](https://docs.rs/tui-spinner)
[![Downloads](https://img.shields.io/crates/d/tui-spinner)](https://crates.io/crates/tui-spinner)
[![MIT License](https://img.shields.io/badge/license-MIT-blue.svg)](LICENSE)

Customizable spinner widgets for [Ratatui](https://github.com/ratatui/ratatui) TUI applications.

---

## Widgets

| Widget | Shape | Motion | Key options |
|--------|-------|--------|-------------|
| [`LinearSpinner`](#linearspinner) | Straight line | Scrolling window or bouncing dot | direction, flow, style |
| [`SquareSpinner`](#squarespinner) | Square braille ring | Rotating arc | size, spin, centre |
| [`CircleSpinner`](#circlespinner) | Circular braille ring | Rotating arc | radius, spin, arc_len |
| [`RectSpinner`](#rectspinner) | Rectangle braille ring | Rotating arc | shape, spin, centre |
| [`BarSpinner`](#barspinner) | Solid horizontal bar | Bouncing or continuous sweep | bar_style, motion, spin, track, fade_width |
| [`FluxSpinner`](#fluxspinner) | Single-character glyph | Cycling frame sequence | frames, width, phase_step, spin |

---

## Previews

### SquareSpinner — Filled & Empty · CW ↻ and CCW ↺

![SquareSpinner Demo](examples/vhs/generated/square-spinner-demo.gif)

### CircleSpinner — Clockwise ↻ and Counter-Clockwise ↺

![CircleSpinner Demo](examples/vhs/generated/circle-spinner-demo.gif)

### LinearSpinner — Orientation & Direction

![LinearSpinner Demo](examples/vhs/generated/linear-spinner-demo.gif)

### BarSpinner — Zed / Claude-style bouncing bar

![BarSpinner Demo](examples/vhs/generated/bar-spinner-demo.gif)

### FluxSpinner — All frame presets

![FluxSpinner Demo](examples/vhs/generated/flux-spinner-demo.gif)

### All Widgets — Combined Overview

![Spinner Demo](examples/vhs/generated/spinner-demo.gif)

---

## Installation

```toml
[dependencies]
tui-spinner = "0.2"
```

---

## Quick Start

```rust
use ratatui::style::Color;
use tui_spinner::{
    BarMotion, BarSpinner, BarStyle, Centre, CircleSpinner, Direction, FluxFrames, FluxSpinner,
    Flow, LinearSpinner, LinearStyle, RectShape, RectSpinner, Spin, SquareSpinner,
};

// Vertical bouncing dot (Zed / Copilot style)
let v = LinearSpinner::new(tick)
    .direction(Direction::Vertical)
    .linear_style(LinearStyle::Braille)
    .active_color(Color::Cyan);

// Square arc, clockwise, filled centre
let sq = SquareSpinner::new(tick)
    .size(3)
    .spin(Spin::Clockwise)
    .centre(Centre::Filled)
    .arc_color(Color::Cyan);

// Circular arc, counter-clockwise
let circle = CircleSpinner::new(tick)
    .radius(5)
    .spin(Spin::CounterClockwise)
    .arc_color(Color::Magenta);

// Zed / Claude-style bouncing bar — fills available width automatically
let bar = BarSpinner::new(tick)
    .arc_color(Color::Cyan)
    .dim_color(Color::DarkGray);

// Continuous sweep (Loop motion) with a Star symbol style
let sweep = BarSpinner::new(tick)
    .bar_style(BarStyle::Star)
    .motion(BarMotion::Loop)
    .spin(Spin::Clockwise)
    .arc_color(Color::Yellow);

// Use preset constructors for common configurations
let preset = BarSpinner::claude(tick);   // 2-row orange bar

// Minimal 1×1 status-bar spinner
let flux = FluxSpinner::new(tick)
    .color(Color::Cyan);

// Wave of rotating glyphs
let wave = FluxSpinner::new(tick)
    .frames(FluxFrames::ORBIT)
    .width(8)
    .phase_step(1)
    .color(Color::LightBlue);
```

All widgets are **stateless** — just pass a monotonically-increasing `tick: u64` counter:

```rust
struct App { tick: u64 }

fn update(app: &mut App) {
    app.tick = app.tick.wrapping_add(1);
}
```

---

## Running the Examples

```sh
cargo run --example square_spinner   # SquareSpinner: Filled/Empty × CW/CCW
cargo run --example circle_spinner   # CircleSpinner: CW and CCW columns
cargo run --example linear_spinner   # LinearSpinner: all styles × orientation × direction
cargo run --example bar_spinner      # BarSpinner: Zed/Claude-style bouncing bar
cargo run --example flux_spinner     # FluxSpinner: all frame presets grid
cargo run --example spinner          # Combined overview of all widgets
```

---

## Widget Reference

### `LinearSpinner`

Horizontal scrolling window or vertical bouncing dot.

```text
Horizontal (Forwards):   ●●·  →  ·●●  →  ··●  →  ●··  → …
Horizontal (Backwards):  ··●  →  ·●●  →  ●●·  →  ●··  → …
Vertical:                ●       ·       ·
                         ·   →   ●   →   ·   → bounces back
                         ·       ·       ●
```

| Builder | Default | Description |
|---------|---------|-------------|
| `direction(Direction)` | `Horizontal` | Animation axis |
| `flow(Flow)` | `Forwards` | Animation direction |
| `linear_style(LinearStyle)` | `Classic` | Symbol set for active/inactive slots |
| `total_slots(n)` | `3` | Total number of slots |
| `lit_slots(n)` | `2` | Simultaneously lit slots (horizontal only) |
| `ticks_per_step(n)` | `3` | Ticks per step (higher = slower) |
| `active_color(c)` | `White` | Lit symbol colour |
| `inactive_color(c)` | `DarkGray` | Dim symbol colour |
| `block(b)` | — | Optional `Block` wrapper |
| `style(s)` | — | Base widget style |

#### `Direction`

| Variant | Description |
|---------|-------------|
| `Horizontal` | Scrolling window across a row (default) |
| `Vertical` | Single dot bouncing up and down a column |

#### `Flow`

| Variant | Description |
|---------|-------------|
| `Forwards` | Normal direction — left-to-right / upward-first (default) |
| `Backwards` | Reversed — right-to-left / downward-first |

#### `LinearStyle`

| Variant | Active | Inactive |
|---------|--------|----------|
| `Classic` | `●` | `·` |
| `Square` | `■` | `□` |
| `Diamond` | `◆` | `◇` |
| `Bar` | `┃` | `│` |
| `Braille` | `⣿` | `⠿` |
| `Arrow` | `▶`/`▼` | `▷`/`▽` |

---

### `SquareSpinner`

Braille-dot arc rotating around a square ring, with optional filled centre.

| Builder | Default | Description |
|---------|---------|-------------|
| `size(n)` | `2` | Arc thickness / square size (2–8) |
| `spin(Spin)` | `Clockwise` | Rotation direction |
| `centre(Centre)` | `Filled` | Interior fill mode |
| `arc_color(c)` | `White` | Rotating arc colour |
| `dim_color(c)` | `DarkGray` | Centre fill / dim ring colour |
| `ticks_per_step(n)` | `1` | Ticks per arc step |
| `alignment(a)` | `Left` | Horizontal alignment |
| `block(b)` | — | Optional `Block` wrapper |

> **Note:** `SquareSpinner` is a convenience wrapper around `RectSpinner`.
> For new code prefer `RectSpinner::new(tick).shape(RectShape::Square(n))`.

---

### `CircleSpinner`

Braille-dot arc rotating around a circular ring computed with the midpoint
circle algorithm.

| Builder | Default | Description |
|---------|---------|-------------|
| `radius(n)` | `4` | Circle radius in braille dots |
| `arc_len(n)` | `0` (auto ¼) | Bright arc length in perimeter dots |
| `spin(Spin)` | `Clockwise` | Rotation direction |
| `ticks_per_step(n)` | `1` | Ticks per arc step |
| `arc_color(c)` | `White` | Bright arc colour |
| `dim_color(c)` | `DarkGray` | Dim ring colour |
| `alignment(a)` | `Left` | Horizontal alignment |
| `block(b)` | — | Optional `Block` wrapper |

Use `.char_size()` → `(cols, rows)` to query the exact rendered size.

---

### `RectSpinner`

Braille-dot arc rotating around a configurable rectangle.

| Builder | Default | Description |
|---------|---------|-------------|
| `shape(RectShape)` | `Square(2)` | Shape and size |
| `spin(Spin)` | `Clockwise` | Rotation direction |
| `centre(Centre)` | `Filled` | Interior fill mode |
| `outer_color(c)` | `Cyan` | Rotating arc colour |
| `inner_color(c)` | `DarkGray` | Centre fill colour |
| `ticks_per_step(n)` | `1` | Ticks per arc step |
| `alignment(a)` | `Left` | Horizontal alignment |
| `block(b)` | — | Optional `Block` wrapper |

#### `RectShape`

| Variant | Description |
|---------|-------------|
| `Square(n)` | Square output; `n` = arc thickness / size (2–8) |

---

### `Spin` (shared)

| Variant | Description |
|---------|-------------|
| `Clockwise` | Arc travels clockwise (default) |
| `CounterClockwise` | Arc travels counter-clockwise |

### `Centre` (shared)

| Variant | Description |
|---------|-------------|
| `Filled` | Solid interior block (default) |
| `Empty` | No interior fill |

---

### `BarSpinner`

A solid braille or symbol bar with a glowing arc that **bounces** (ping-pong) or **sweeps continuously** (loop).  The arc edges taper through a density ramp for a soft comet-glow look.

Set `width(0)` (the default) to fill the available area automatically.

#### Preset constructors

```rust
BarSpinner::zed(tick)       // 1 row · cyan   · Rail track
BarSpinner::claude(tick)    // 2 rows · orange · Rail track
BarSpinner::minimal(tick)   // 1 row · white  · Empty track (arc floats)
BarSpinner::solid(tick)     // 1 row · cyan   · Full track · fade=0
```

#### Builder reference

| Builder | Default | Description |
|---------|---------|-------------|
| `width(n)` | `0` (auto-fill) | Fixed column count; `0` = fill area |
| `height(n)` | `1` | Bar height in rows |
| `arc_width(n)` | `0` (auto ⅓) | Bright arc width in character columns |
| `spin(Spin)` | `Clockwise` | Starting direction |
| `motion(BarMotion)` | `Bounce` | Edge behaviour: bounce or loop |
| `bar_style(BarStyle)` | `Braille` | Glyph set for arc and track |
| `track(BarTrack)` | `Rail` | Background track appearance |
| `fade_width(n)` | `3` | Gradient ramp width (0 = sharp cutoff) |
| `arc_char(byte)` | `0xFF` (`⣿`) | Braille byte for arc centre cells |
| `arc_color(c)` | `Cyan` | Bright arc colour |
| `dim_color(c)` | `DarkGray` | Background track colour |
| `with_colors(arc, dim)` | — | Set both colours in one call |
| `ticks_per_step(n)` | `1` | Ticks per step (higher = slower) |
| `alignment(a)` | `Left` | Horizontal alignment |
| `block(b)` | — | Optional `Block` wrapper |

#### `BarMotion`

| Variant | Description |
|---------|-------------|
| `Bounce` | Reverses at each edge — ping-pong (default) |
| `Loop` | Wraps around: exits one edge, re-enters from the other |

#### `BarStyle`

| Variant | Arc | Track |
|---------|-----|-------|
| `Braille` | `⣿` | `⣀` |
| `Block` | `█` | `░` |
| `Shade` | `▓` | `░` |
| `Dot` | `●` | `·` |
| `Diamond` | `◆` | `◇` |
| `Square` | `■` | `□` |
| `Star` | `★` | `☆` |
| `Heart` | `♥` | `♡` |
| `Arrow` | `▶` | `▷` |
| `Circle` | `◉` | `○` |
| `Spark` | `✦` | `✧` |
| `Cross` | `✚` | `✛` |
| `Progress` | `▰` | `▱` |
| `Thick` | `━` | `─` |
| `Wave` | `≈` | `˜` |
| `Pip` | `▪` | `·` |

#### `BarTrack`

| Variant | Byte | Glyph | Effect |
|---------|------|-------|--------|
| `Rail` | `0xC0` | `⣀` | Bottom-two-dot baseline (default) |
| `Full` | `0xFF` | `⣿` | Full-density track |
| `Empty` | `0x00` | `⠀` | Invisible — arc floats |
| `Custom(u8)` | any | — | User-defined braille byte |

---

### `FluxSpinner`

A compact single-character spinner that cycles through a frame sequence.
At `1×1` it is a minimal status-bar indicator; scaled up with
[`width`](FluxSpinner::width) / [`height`](FluxSpinner::height) it
produces a travelling diagonal wave controlled by
[`phase_step`](FluxSpinner::phase_step).

| Builder | Default | Description |
|---------|---------|-------------|
| `frames(f)` | `FluxFrames::BRAILLE` | Frame sequence (any `&'static [char]`) |
| `width(n)` | `1` | Width in character columns |
| `height(n)` | `1` | Height in character rows |
| `spin(Spin)` | `Clockwise` | Frame sequence direction |
| `ticks_per_step(n)` | `1` | Ticks per frame (higher = slower) |
| `phase_step(n)` | `1` | Frame offset between adjacent cells |
| `color(c)` | `Cyan` | Glyph colour |
| `alignment(a)` | `Left` | Horizontal alignment |
| `block(b)` | — | Optional `Block` wrapper |

#### `FluxFrames` presets

| Preset | Glyphs | Frames | Description |
|--------|--------|--------|-------------|
| `BRAILLE` | `⣾ ⣷ ⣯ ⣟ ⡿ ⢿ ⣽ ⣻` | 8 | Full cell, one dot missing (default) |
| `ORBIT` | `⠁ ⠈ ⠐ ⠠ ⢀ ⡀ ⠄ ⠂` | 8 | Single dot orbiting |
| `CLASSIC` | `⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏` | 10 | Classic braille spinner |
| `LINE` | `│ ╱ ─ ╲` | 4 | Rotating line |
| `BLOCK` | `▖ ▘ ▝ ▗` | 4 | Quarter-block rotation |
| `ARC` | `◜ ◝ ◞ ◟` | 4 | Quarter-arc rotation |
| `CLOCK` | `◷ ◶ ◵ ◴` | 4 | Quarter-circle pie slice |
| `MOON` | `◓ ◑ ◒ ◐` | 4 | Half-circle moon phase |
| `TRIANGLES` | `▲ ▶ ▼ ◀` | 4 | Filled triangle rotation |
| `PULSE` | `⣀ ⣤ ⣶ ⣾ ⣿ ⣾ ⣶ ⣤` | 8 | Braille fill pulse |
| `BOUNCE` | `⠉ ⠒ ⣀ ⠒` | 4 | Braille dot bouncing |
| `HALF` | `▀ ▐ ▄ ▌` | 4 | Half-block rotation |
| `SQUARE` | `◰ ◳ ◲ ◱` | 4 | Filled square quadrant |
| `DICE` | `⚀ ⚁ ⚂ ⚃ ⚄ ⚅` | 6 | Dice faces |
| `BAR` | `▁ ▂ ▃ ▄ ▅ ▆ ▇ █` | 8 | Growing bar |
| `CORNERS` | `┌ ┐ ┘ └` | 4 | Box corners |
| `CIRCLE_FILL` | `○ ◔ ◑ ◕ ●` | 5 | Circle filling |
| `PISTON` | `▁ ▃ ▅ ▇ █ ▇ ▅ ▃` | 8 | Bouncing bar |
| `STAR` | `✶ ✷ ✸ ✹` | 4 | Star density ramp |
| `PAIR` | `⠉ ⠘ ⠰ ⢠ ⣀ ⡄ ⠆ ⠃` | 8 | Two dots rotating together |
| `DIAMOND` | `◇ ◈ ◆ ◈` | 4 | Diamond pulse |

Pass **any** `&'static [char]` for a fully custom animation:

```rust
let custom = FluxSpinner::new(tick)
    .frames(&['◐', '◓', '◑', '◒'])
    .color(Color::Cyan);
```

#### `phase_step` wave effect

```text
width = 6, phase_step = 1, Clockwise
⣾⣷⣯⣟⡿⢿   (tick 0)
⣷⣯⣟⡿⢿⣽   (tick 1)
⣯⣟⡿⢿⣽⣻   (tick 2)  …
```

With `Spin::CounterClockwise` the wave travels in the opposite direction.

---

## Generating Demo GIFs

```sh
# Requires VHS: https://github.com/charmbracelet/vhs
vhs examples/vhs/square-spinner-demo.tape
vhs examples/vhs/circle-spinner-demo.tape
vhs examples/vhs/linear-spinner-demo.tape
vhs examples/vhs/bar-spinner-demo.tape
vhs examples/vhs/flux-spinner-demo.tape
vhs examples/vhs/spinner-demo.tape

# Or via justfile:
just vhs-all
```

GIF files are tracked with **Git LFS** (see `.gitattributes`).

---

## License

MIT — see [LICENSE](LICENSE).