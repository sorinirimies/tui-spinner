//! # FluxSpinner Example
//!
//! Displays all [`FluxFrames`] presets in a compact **5 × 5 grid** (21 presets +
//! 4 live custom-frames tiles = 25 cells).
//!
//! Each tile shows:
//! - The preset name as the border title
//! - The **CW ↻** and **CCW ↺** animated glyphs side by side
//! - The frame count
//!
//! The four **CUSTOM** tiles demonstrate passing any `&'static [char]` slice
//! directly to [`.frames()`](tui_spinner::FluxSpinner::frames).
//!
//! **Controls:** `q` / `Esc` — Quit
//!
//! Run with: `cargo run --example flux_spinner`

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Paragraph},
    DefaultTerminal, Frame,
};
use std::time::{Duration, Instant};
use tui_spinner::{FluxFrames, Spin};

// ── Style macros ──────────────────────────────────────────────────────────────

/// Build a [`Style`] quickly.
///
/// ```text
/// sty!(dim)              → Style::default().fg(Color::DarkGray)
/// sty!(Color::Cyan)      → Style::default().fg(Color::Cyan)
/// sty!(Color::Cyan, b)   → …fg(Cyan).add_modifier(BOLD)
/// ```
macro_rules! sty {
    (dim) => {
        Style::default().fg(Color::DarkGray)
    };
    ($c:expr) => {
        Style::default().fg($c)
    };
    ($c:expr, b) => {
        Style::default().fg($c).add_modifier(Modifier::BOLD)
    };
}

/// Build a styled [`Span`] quickly.
///
/// ```text
/// sp!("·"; dim)               → dim grey span
/// sp!("text"; Color::Cyan)    → cyan span
/// sp!("text"; Color::Cyan, b) → bold cyan span
/// ```
macro_rules! sp {
    ($t:expr; dim) => {
        Span::styled($t, sty!(dim))
    };
    ($t:expr; $c:expr) => {
        Span::styled($t, sty!($c))
    };
    ($t:expr; $c:expr, b) => {
        Span::styled($t, sty!($c, b))
    };
}

// ── Tile data ─────────────────────────────────────────────────────────────────

/// One grid cell: the frame sequence, display name, accent colour, and a flag
/// that marks it as a live custom-frames demo rather than a named preset.
#[derive(Copy, Clone)]
struct Tile {
    frames: &'static [char],
    name: &'static str,
    color: Color,
    custom: bool,
}

/// Define a named-preset tile.
macro_rules! preset {
    ($frames:expr, $name:literal, $color:expr) => {
        Tile {
            frames: $frames,
            name: $name,
            color: $color,
            custom: false,
        }
    };
}

/// Define a live custom-frames demo tile.
macro_rules! custom {
    ($frames:expr, $name:literal, $color:expr) => {
        Tile {
            frames: $frames,
            name: $name,
            color: $color,
            custom: true,
        }
    };
}

// ── Tile table ────────────────────────────────────────────────────────────────
//
// 16 named presets + 4 custom demos = 20 tiles → 5 rows × 4 cols.

static TILES: &[Tile] = &[
    // Row 1
    preset!(FluxFrames::BRAILLE, "BRAILLE", Color::Cyan),
    preset!(FluxFrames::ORBIT, "ORBIT", Color::LightBlue),
    preset!(FluxFrames::CLASSIC, "CLASSIC", Color::White),
    preset!(FluxFrames::LINE, "LINE", Color::Yellow),
    // Row 2
    preset!(FluxFrames::BLOCK, "BLOCK", Color::LightGreen),
    preset!(FluxFrames::ARC, "ARC", Color::LightCyan),
    preset!(FluxFrames::CORNERS, "CORNERS", Color::Rgb(200, 200, 100)),
    preset!(FluxFrames::CLOCK, "CLOCK", Color::Rgb(200, 160, 255)),
    // Row 3
    preset!(FluxFrames::MOON, "MOON", Color::Rgb(180, 220, 255)),
    preset!(FluxFrames::TRIANGLES, "TRIANGLES", Color::LightMagenta),
    preset!(FluxFrames::PULSE, "PULSE", Color::LightRed),
    preset!(FluxFrames::BOUNCE, "BOUNCE", Color::Rgb(255, 180, 80)),
    // Row 4
    preset!(FluxFrames::HALF, "HALF", Color::Rgb(120, 220, 180)),
    preset!(FluxFrames::SQUARE, "SQUARE", Color::Rgb(255, 140, 200)),
    preset!(FluxFrames::DICE, "DICE", Color::Rgb(255, 220, 100)),
    preset!(FluxFrames::BAR, "BAR", Color::Rgb(160, 255, 160)),
    preset!(
        FluxFrames::CIRCLE_FILL,
        "CIRCLEFILL",
        Color::Rgb(100, 210, 255)
    ),
    // Row 5
    preset!(FluxFrames::PISTON, "PISTON", Color::Rgb(255, 160, 80)),
    preset!(FluxFrames::STAR, "STAR", Color::Rgb(255, 240, 120)),
    preset!(FluxFrames::PAIR, "PAIR", Color::Rgb(140, 255, 200)),
    preset!(FluxFrames::DIAMOND, "DIAMOND", Color::Rgb(255, 160, 220)),
    // Row 5 col 5 + Row 6 — live custom-frames demos
    custom!(&['○', '◎', '●', '◎'], "RINGS", Color::Rgb(100, 200, 255)),
    custom!(
        &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'],
        "ALPHA",
        Color::Rgb(180, 255, 180)
    ),
    custom!(&['░', '▒', '▓', '█'], "SHADE", Color::Rgb(200, 180, 255)),
    custom!(&['♩', '♪', '♫', '♬'], "MUSIC", Color::Rgb(255, 180, 220)),
];

const GRID_ROWS: usize = 4;
const GRID_COLS: usize = 7;
/// Each cell is exactly this many terminal rows tall: 2 borders + 2 content.
const CELL_H: u16 = 4;
/// Ticks held per animation step — raise to slow animation down.
const TPS: u64 = 4;

// ── App ───────────────────────────────────────────────────────────────────────

struct App {
    tick: u64,
    last_tick: Instant,
}

impl Default for App {
    fn default() -> Self {
        Self {
            tick: 0,
            last_tick: Instant::now(),
        }
    }
}

// ── Entry point ───────────────────────────────────────────────────────────────

fn main() -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();
    let result = run(terminal, &mut App::default());
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        let now = Instant::now();
        let steps = (now.duration_since(app.last_tick).as_millis() / 80).max(1) as u64;
        app.last_tick = now;
        app.tick = app.tick.wrapping_add(steps);

        terminal.draw(|f| render(f, app))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(k) = event::read()? {
                if matches!(k.code, KeyCode::Char('q') | KeyCode::Esc) {
                    break;
                }
            }
        }
    }
    Ok(())
}

// ── Root layout ───────────────────────────────────────────────────────────────

fn render(frame: &mut Frame, app: &App) {
    let [header, body, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    render_header(frame, header);
    render_grid(frame, body, app.tick);
    render_footer(frame, footer);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        sp!("↻"; Color::Cyan),
        sp!("  Clockwise"; Color::Cyan),
        sp!("  ·  frame preset ·  "; dim),
        sp!("Counter-Clockwise  "; Color::Blue),
        sp!("↺"; Color::Blue),
    ]);
    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center).block(
            Block::bordered()
                .title(" FluxSpinner ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(sty!(dim)),
        ),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        sp!("q"; Color::Cyan, b),
        sp!(" / "; dim),
        sp!("Esc"; Color::Cyan, b),
        sp!("  Quit"; dim),
    ]);
    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(sty!(dim)),
        ),
        area,
    );
}

// ── Grid ──────────────────────────────────────────────────────────────────────

fn render_grid(frame: &mut Frame, area: Rect, tick: u64) {
    // Fixed-height rows so cells are always compact regardless of terminal size.
    let row_cs: Vec<Constraint> = (0..GRID_ROWS)
        .map(|_| Constraint::Length(CELL_H))
        .chain(std::iter::once(Constraint::Min(0)))
        .collect();

    let col_cs: Vec<Constraint> = (0..GRID_COLS)
        .map(|_| Constraint::Ratio(1, GRID_COLS as u32))
        .collect();
    // 5 × 5 = 25 cells: 21 presets + 4 custom demos.

    let rows = Layout::vertical(row_cs).split(area);

    for row_idx in 0..GRID_ROWS {
        let cols = Layout::horizontal(col_cs.clone()).split(rows[row_idx]);

        for col_idx in 0..GRID_COLS {
            let idx = row_idx * GRID_COLS + col_idx;
            if let Some(tile) = TILES.get(idx) {
                render_tile(frame, cols[col_idx], tick, tile);
            }
        }
    }
}

// ── Tile ──────────────────────────────────────────────────────────────────────

fn render_tile(frame: &mut Frame, area: Rect, tick: u64, tile: &Tile) {
    // Custom tiles get a dim border; preset tiles get their accent colour.
    let border_style = if tile.custom {
        sty!(dim)
    } else {
        sty!(tile.color)
    };
    let title = if tile.custom {
        format!(" {} · custom ", tile.name)
    } else {
        format!(" {} ", tile.name)
    };

    let block = Block::bordered()
        .title(title)
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(border_style);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height < 2 {
        return;
    }

    // Two content rows: animated glyphs on top, metadata below.
    let [spinner_row, meta_row] =
        Layout::vertical([Constraint::Length(1), Constraint::Length(1)]).areas(inner);

    render_spinners(frame, spinner_row, tick, tile);
    render_meta(frame, meta_row, tile);
}

/// Row 1: `↻ [CW char]    [CCW char] ↺`
fn render_spinners(frame: &mut Frame, area: Rect, tick: u64, tile: &Tile) {
    let cw = frame_char(tile.frames, tick, Spin::Clockwise);
    let ccw = frame_char(tile.frames, tick, Spin::CounterClockwise);

    let line = Line::from(vec![
        sp!("↻ "; dim),
        sp!(cw.to_string(); tile.color, b),
        sp!("    "; dim),
        sp!(ccw.to_string(); tile.color, b),
        sp!(" ↺"; dim),
    ]);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), area);
}

/// Row 2: frame count — or `.frames(&[…])` hint for custom tiles.
fn render_meta(frame: &mut Frame, area: Rect, tile: &Tile) {
    let text = if tile.custom {
        format!(".frames(&[…])  ·  {} frames", tile.frames.len())
    } else {
        format!("· {} frames ·", tile.frames.len())
    };
    frame.render_widget(
        Paragraph::new(sp!(text; dim)).alignment(Alignment::Center),
        area,
    );
}

// ── Frame helper ──────────────────────────────────────────────────────────────

/// Return the current animated glyph for `frames` at `tick` in `spin` direction.
///
/// Mirrors the index arithmetic inside [`tui_spinner::FluxSpinner`]'s renderer.
fn frame_char(frames: &[char], tick: u64, spin: Spin) -> char {
    let n = frames.len();
    if n == 0 {
        return ' ';
    }
    let base = (tick / TPS) as usize;
    let idx = match spin {
        Spin::CounterClockwise => (n - base % n) % n,
        _ => base % n,
    };
    frames[idx]
}
