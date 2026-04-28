//! # FluxSpinner Example
//!
//! Displays all [`FluxFrames`] presets in a **4 × 3 grid**.
//! Every tile shows the preset name, the CW ↻ and CCW ↺ animated glyphs
//! side by side, the full frame sequence, and the frame count.
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
        FluxFrames::ARROWS,
        "ARROWS",
        "↑ ↗ → ↘ ↓ ↙ ← ↖",
        Color::LightYellow,
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

// ── 4 × 3 grid ────────────────────────────────────────────────────────────────

fn render_grid(frame: &mut Frame, area: Rect, tick: u64) {
    let rows = Layout::vertical([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .split(area);

    for (row_idx, &row_area) in rows.iter().enumerate() {
        let cols = Layout::horizontal([
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
            Constraint::Ratio(1, 4),
        ])
        .split(row_area);

        for (col_idx, &col_area) in cols.iter().enumerate() {
            let preset_idx = row_idx * 4 + col_idx;
            if preset_idx < PRESETS.len() {
                let (frames, name, glyphs, color) = PRESETS[preset_idx];
                render_tile(frame, col_area, tick, frames, name, glyphs, color);
            } else if preset_idx == 11 {
                render_custom_tile(frame, col_area);
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

    // Divide inner into three equal horizontal bands.
    let [spinners_area, glyphs_area, count_area] = Layout::vertical([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .areas(inner);

    let cw = frame_char(frames, tick, Spin::Clockwise);
    let ccw = frame_char(frames, tick, Spin::CounterClockwise);
    let dim = Style::default().fg(Color::DarkGray);
    let accent = Style::default().fg(color);
    let bold_accent = accent.add_modifier(Modifier::BOLD);

    // Band 1 — animated CW and CCW glyphs.
    let spinner_line = Line::from(vec![
        Span::styled("↻ ", dim),
        Span::styled(cw.to_string(), bold_accent),
        Span::styled("   ", dim),
        Span::styled(ccw.to_string(), bold_accent),
        Span::styled(" ↺", dim),
    ]);
    render_vcenter(frame, spinners_area, spinner_line);

    // Band 2 — full frame sequence.
    render_vcenter(frame, glyphs_area, Line::from(Span::styled(glyphs, accent)));

    // Band 3 — frame count.
    render_vcenter(
        frame,
        count_area,
        Line::from(Span::styled(format!("· {} frames ·", frames.len()), dim)),
    );
}

fn render_custom_tile(frame: &mut Frame, area: Rect) {
    let dim = Style::default().fg(Color::DarkGray);

    let block = Block::bordered()
        .title(" CUSTOM ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(dim);

    let inner = block.inner(area);
    frame.render_widget(block, area);

    let [top, mid, bot] = Layout::vertical([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .areas(inner);

    render_vcenter(
        frame,
        top,
        Line::from(Span::styled("any &'static [char]", dim)),
    );
    render_vcenter(
        frame,
        mid,
        Line::from(Span::styled(
            ".frames(&['a','b','c'])",
            Style::default()
                .fg(Color::DarkGray)
                .add_modifier(Modifier::ITALIC),
        )),
    );
    render_vcenter(frame, bot, Line::from(Span::styled("· any length ·", dim)));
}

// ── Helpers ───────────────────────────────────────────────────────────────────

/// Render `line` centred horizontally and vertically within `area`.
fn render_vcenter(frame: &mut Frame, area: Rect, line: Line) {
    let [_, center, _] = Layout::vertical([
        Constraint::Min(0),
        Constraint::Length(1),
        Constraint::Min(0),
    ])
    .areas(area);
    frame.render_widget(Paragraph::new(line).alignment(Alignment::Center), center);
}
