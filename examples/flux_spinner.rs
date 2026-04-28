//! # FluxSpinner Example
//!
//! Displays all [`FluxFrames`] presets in a compact **5 × 4 grid** (16 presets +
//! 4 live custom-frames tiles).  Every tile shows the preset name, the
//! CW ↻ and CCW ↺ animated glyphs side by side, the full frame sequence,
//! and the frame count.
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

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut app = App::default();
    let terminal = ratatui::init();
    let result = run(terminal, &mut app);
    ratatui::restore();
    result
}

fn run(mut terminal: DefaultTerminal, app: &mut App) -> Result<()> {
    loop {
        let now = Instant::now();
        let delta = now.duration_since(app.last_tick);
        app.last_tick = now;
        let steps = (delta.as_millis() / 80).max(1) as u64;
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

// ── Frame helper ──────────────────────────────────────────────────────────────

/// Ticks held per animation step — increase to slow the animation down.
const TPS: u64 = 4;

/// Return the current animated glyph for `frames` at `tick` in `spin` direction.
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

// ── Preset table ──────────────────────────────────────────────────────────────

/// (frames, name, glyph-sequence preview, accent color)
const PRESETS: &[(&[char], &str, &str, Color)] = &[
    (
        FluxFrames::BRAILLE,
        "BRAILLE",
        "⣾ ⣷ ⣯ ⣟ ⡿ ⢿ ⣽ ⣻",
        Color::Cyan,
    ),
    (
        FluxFrames::ORBIT,
        "ORBIT",
        "⠁ ⠈ ⠐ ⠠ ⢀ ⡀ ⠄ ⠂",
        Color::LightBlue,
    ),
    (
        FluxFrames::CLASSIC,
        "CLASSIC",
        "⠋ ⠙ ⠹ ⠸ ⠼ ⠴ ⠦ ⠧ ⠇ ⠏",
        Color::White,
    ),
    (FluxFrames::LINE, "LINE", "│ ╱ ─ ╲", Color::Yellow),
    (FluxFrames::BLOCK, "BLOCK", "▖ ▘ ▝ ▗", Color::LightGreen),
    (FluxFrames::ARC, "ARC", "◜ ◝ ◞ ◟", Color::LightCyan),
    (
        FluxFrames::CORNERS,
        "CORNERS",
        "┌ ┐ ┘ └",
        Color::Rgb(200, 200, 100),
    ),
    (
        FluxFrames::CLOCK,
        "CLOCK",
        "◷ ◶ ◵ ◴",
        Color::Rgb(200, 160, 255),
    ),
    (
        FluxFrames::MOON,
        "MOON",
        "◓ ◑ ◒ ◐",
        Color::Rgb(180, 220, 255),
    ),
    (
        FluxFrames::TRIANGLES,
        "TRIANGLES",
        "▲ ▶ ▼ ◀",
        Color::LightMagenta,
    ),
    (
        FluxFrames::PULSE,
        "PULSE",
        "⣀ ⣤ ⣶ ⣾ ⣿ ⣾ ⣶ ⣤",
        Color::LightRed,
    ),
    (
        FluxFrames::BOUNCE,
        "BOUNCE",
        "⠉ ⠒ ⣀ ⠒",
        Color::Rgb(255, 180, 80),
    ),
    (
        FluxFrames::HALF,
        "HALF",
        "▀ ▐ ▄ ▌",
        Color::Rgb(120, 220, 180),
    ),
    (
        FluxFrames::SQUARE,
        "SQUARE",
        "◰ ◳ ◲ ◱",
        Color::Rgb(255, 140, 200),
    ),
    (
        FluxFrames::DICE,
        "DICE",
        "⚀ ⚁ ⚂ ⚃ ⚄ ⚅",
        Color::Rgb(255, 220, 100),
    ),
    (
        FluxFrames::BAR,
        "BAR",
        "▁ ▂ ▃ ▄ ▅ ▆ ▇ █",
        Color::Rgb(160, 255, 160),
    ),
];

/// Live custom-frame tiles — demonstrate `.frames(&[…])` with arbitrary slices.
const CUSTOM_TILES: &[(&[char], &str, &str, Color)] = &[
    (
        &['○', '◎', '●', '◎'],
        "RINGS",
        "○ ◎ ● ◎",
        Color::Rgb(100, 200, 255),
    ),
    (
        &['a', 'b', 'c', 'd', 'e', 'f', 'g', 'h'],
        "ALPHA",
        "a b c d e f g h",
        Color::Rgb(180, 255, 180),
    ),
    (
        &['░', '▒', '▓', '█'],
        "SHADE",
        "░ ▒ ▓ █",
        Color::Rgb(200, 180, 255),
    ),
    (
        &['♩', '♪', '♫', '♬'],
        "MUSIC",
        "♩ ♪ ♫ ♬",
        Color::Rgb(255, 180, 220),
    ),
];

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
    let dim = Style::default().fg(Color::DarkGray);
    let line = Line::from(vec![
        Span::styled("↻", Style::default().fg(Color::Cyan)),
        Span::styled("  Clockwise", Style::default().fg(Color::Cyan)),
        Span::styled("  ·  frame preset ·  ", dim),
        Span::styled("Counter-Clockwise  ", Style::default().fg(Color::Blue)),
        Span::styled("↺", Style::default().fg(Color::Blue)),
    ]);
    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center).block(
            Block::bordered()
                .title(" FluxSpinner ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(dim),
        ),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let line = Line::from(vec![
        Span::styled(
            "q",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw(" / "),
        Span::styled(
            "Esc",
            Style::default()
                .fg(Color::Cyan)
                .add_modifier(Modifier::BOLD),
        ),
        Span::raw("  Quit"),
    ]);
    frame.render_widget(
        Paragraph::new(line).alignment(Alignment::Center).block(
            Block::bordered()
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
        ),
        area,
    );
}

// ── 5 × 4 compact grid ────────────────────────────────────────────────────────
//
// Each cell row is exactly 5 terminal rows tall (2 borders + 3 content rows).
// Remaining vertical space sits below the grid.

const GRID_ROWS: usize = 5;
const GRID_COLS: usize = 4;

fn render_grid(frame: &mut Frame, area: Rect, tick: u64) {
    // Fixed-height rows so cells are always compact regardless of terminal size.
    let row_constraints: Vec<Constraint> = (0..GRID_ROWS)
        .map(|_| Constraint::Length(5))
        .chain(std::iter::once(Constraint::Min(0)))
        .collect();

    let col_constraints: Vec<Constraint> = (0..GRID_COLS)
        .map(|_| Constraint::Ratio(1, GRID_COLS as u32))
        .collect();

    let rows = Layout::vertical(row_constraints).split(area);

    for row_idx in 0..GRID_ROWS {
        let cols = Layout::horizontal(col_constraints.clone()).split(rows[row_idx]);

        for col_idx in 0..GRID_COLS {
            let col_area = cols[col_idx];
            let idx = row_idx * GRID_COLS + col_idx;

            if idx < PRESETS.len() {
                let (frames, name, glyphs, color) = PRESETS[idx];
                render_tile(frame, col_area, tick, frames, name, glyphs, color);
            } else {
                let custom_idx = idx - PRESETS.len();
                if custom_idx < CUSTOM_TILES.len() {
                    let (frames, name, glyphs, color) = CUSTOM_TILES[custom_idx];
                    render_custom_tile(frame, col_area, tick, frames, name, glyphs, color);
                }
            }
        }
    }
}

// ── Tile ──────────────────────────────────────────────────────────────────────

fn render_tile(
    frame: &mut Frame,
    area: Rect,
    tick: u64,
    frames: &[char],
    name: &str,
    glyphs: &str,
    color: Color,
) {
    let block = Block::bordered()
        .title(format!(" {name} "))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(color));

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    // Three compact 1-row content bands — no padding needed at cell height 5.
    let [spinner_row, glyph_row, count_row, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(inner);

    let cw = frame_char(frames, tick, Spin::Clockwise);
    let ccw = frame_char(frames, tick, Spin::CounterClockwise);
    let dim = Style::default().fg(Color::DarkGray);
    let accent = Style::default().fg(color);
    let bold_accent = accent.add_modifier(Modifier::BOLD);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("↻ ", dim),
            Span::styled(cw.to_string(), bold_accent),
            Span::styled("  ", dim),
            Span::styled(ccw.to_string(), bold_accent),
            Span::styled(" ↺", dim),
        ]))
        .alignment(Alignment::Center),
        spinner_row,
    );

    frame.render_widget(
        Paragraph::new(Span::styled(glyphs, accent)).alignment(Alignment::Center),
        glyph_row,
    );

    frame.render_widget(
        Paragraph::new(Span::styled(format!("· {} frames ·", frames.len()), dim))
            .alignment(Alignment::Center),
        count_row,
    );
}

fn render_custom_tile(
    frame: &mut Frame,
    area: Rect,
    tick: u64,
    frames: &'static [char],
    name: &str,
    glyphs: &str,
    color: Color,
) {
    let dim = Style::default().fg(Color::DarkGray);

    let block = Block::bordered()
        .title(format!(" {name} · custom "))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(dim);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    if inner.height == 0 {
        return;
    }

    let [spinner_row, glyph_row, hint_row, _] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(inner);

    let cw = frame_char(frames, tick, Spin::Clockwise);
    let ccw = frame_char(frames, tick, Spin::CounterClockwise);
    let accent = Style::default().fg(color);
    let bold_accent = accent.add_modifier(Modifier::BOLD);

    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("↻ ", dim),
            Span::styled(cw.to_string(), bold_accent),
            Span::styled("  ", dim),
            Span::styled(ccw.to_string(), bold_accent),
            Span::styled(" ↺", dim),
        ]))
        .alignment(Alignment::Center),
        spinner_row,
    );

    frame.render_widget(
        Paragraph::new(Span::styled(glyphs, accent)).alignment(Alignment::Center),
        glyph_row,
    );

    frame.render_widget(
        Paragraph::new(Span::styled(
            format!(".frames(&[…]) · {} frames", frames.len()),
            dim,
        ))
        .alignment(Alignment::Center),
        hint_row,
    );
}
