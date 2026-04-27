//! # FluxSpinner Example
//!
//! One row per [`FluxFrames`] preset.  Each row shows the same preset
//! animating **forward** (Clockwise ↻) on the left and **backward**
//! (Counter-Clockwise ↺) on the right so the direction reversal is
//! immediately obvious.
//!
//! **Controls:**
//! - `q` / `Esc` — Quit
//!
//! Run with: `cargo run --example flux_spinner`

use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode};
use ratatui::{
    layout::{Alignment, Constraint, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType, Padding, Paragraph},
    DefaultTerminal, Frame,
};
use std::time::{Duration, Instant};
use tui_spinner::{FluxFrames, FluxSpinner, Spin};

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

        terminal.draw(|frame| render(frame, app))?;

        if event::poll(Duration::from_millis(16))? {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('q') | KeyCode::Esc => break,
                    _ => {}
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
    render_body(frame, body, app.tick);
    render_footer(frame, footer);
}

fn render_header(frame: &mut Frame, area: Rect) {
    frame.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled("↻  Clockwise", Style::default().fg(Color::Cyan)),
            Span::styled(
                "  ·  frame preset  ·  ",
                Style::default().fg(Color::DarkGray),
            ),
            Span::styled("Counter-Clockwise  ↺", Style::default().fg(Color::Blue)),
        ]))
        .alignment(Alignment::Center)
        .block(
            Block::bordered()
                .title(" FluxSpinner ")
                .title_alignment(Alignment::Center)
                .border_type(BorderType::Rounded)
                .border_style(Style::default().fg(Color::DarkGray)),
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

// ── Body ──────────────────────────────────────────────────────────────────────
//
// One row per preset.  Layout of each row:
//
//   [CW spinner]  [name · n frames · glyph sequence]  [CCW spinner]
//
// The spinner column is narrow (1 char); the label fills the middle;
// the CCW spinner mirrors the right edge.

/// (frames, name, frame-count label, glyph-sequence preview, accent color)
const ROWS: &[(&[char], &str, &str, Color)] = &[
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
        "◐ ◓ ◑ ◒",
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

fn render_body(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(2));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = ROWS.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(frames, name, glyphs, color)) in ROWS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        render_preset_row(frame, rows[i], tick, frames, name, glyphs, color);
    }
}

fn render_preset_row(
    frame: &mut Frame,
    area: Rect,
    tick: u64,
    frames: &'static [char],
    name: &str,
    glyphs: &str,
    color: Color,
) {
    // [CW 1ch] [gap 2ch] [label fills] [gap 2ch] [CCW 1ch]
    let [cw_area, gap_l, label_area, gap_r, ccw_area] = Layout::horizontal([
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Min(0),
        Constraint::Length(2),
        Constraint::Length(1),
    ])
    .areas(area);

    // Silence unused-variable warnings on gap areas
    let _ = (gap_l, gap_r);

    // CW spinner
    frame.render_widget(
        FluxSpinner::new(tick)
            .frames(frames)
            .color(color)
            .ticks_per_step(4),
        cw_area,
    );

    // CCW spinner
    frame.render_widget(
        FluxSpinner::new(tick)
            .frames(frames)
            .spin(Spin::CounterClockwise)
            .color(color)
            .ticks_per_step(4),
        ccw_area,
    );

    // Label: "NAME · n frames · glyph sequence"
    let n_frames = frames.len();
    let label = Line::from(vec![
        Span::styled(
            format!("{name:<10}"),
            Style::default().fg(color).add_modifier(Modifier::BOLD),
        ),
        Span::styled("  ·  ", Style::default().fg(Color::DarkGray)),
        Span::styled(
            format!("{n_frames} frames"),
            Style::default().fg(Color::DarkGray),
        ),
        Span::styled("  ·  ", Style::default().fg(Color::DarkGray)),
        Span::styled(glyphs, Style::default().fg(color)),
    ]);

    frame.render_widget(
        Paragraph::new(label).alignment(Alignment::Center),
        label_area,
    );
}
