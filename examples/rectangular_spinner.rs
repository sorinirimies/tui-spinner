//! # RectangularSpinner Example
//!
//! Demonstrates the [`RectangularSpinner`] widget — a Zed / Claude-style
//! braille-dot arc that bounces back and forth along a rectangle perimeter.
//!
//! Showcases:
//! - **Col 1** — Compact spinners (width 6–10, height 2–3) — "Zed style"
//! - **Col 2** — Wide banner spinners (width 16–20, height 2) — "Claude style"
//! - **Col 3** — Tall spinners (height 4–5) with both CW and CCW starts
//! - **Col 4** — Custom arc lengths and speeds
//!
//! **Controls:**
//! - `q` / `Esc` — Quit
//!
//! Run with: `cargo run --example rectangular_spinner`

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
use tui_spinner::{RectangularSpinner, Spin};

// ── App state ─────────────────────────────────────────────────────────────────

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

// ── Top-level layout ──────────────────────────────────────────────────────────

fn render(frame: &mut Frame, app: &App) {
    let [header, content, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    render_header(frame, header);
    render_content(frame, content, app.tick);
    render_footer(frame, footer);
}

fn render_header(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .title(" RectangularSpinner — Zed / Claude-style bouncing arc ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::horizontal(1));

    let text = Paragraph::new("Compact · Wide Banner · Tall · Custom Arc")
        .alignment(Alignment::Center)
        .style(Style::default().fg(Color::Gray));

    frame.render_widget(text.block(block), area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));

    let text = Line::from(vec![
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
        Paragraph::new(text)
            .alignment(Alignment::Center)
            .block(block),
        area,
    );
}

fn render_content(frame: &mut Frame, area: Rect, tick: u64) {
    let [col_compact, col_wide, col_tall, col_custom] = Layout::horizontal([
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
        Constraint::Percentage(25),
    ])
    .areas(area);

    render_compact_column(frame, col_compact, tick);
    render_wide_column(frame, col_wide, tick);
    render_tall_column(frame, col_tall, tick);
    render_custom_column(frame, col_custom, tick);
}

// ── Col 1 — Compact spinners (Zed style) ─────────────────────────────────────

fn render_compact_column(frame: &mut Frame, area: Rect, tick: u64) {
    let outer_block = Block::bordered()
        .title(" Compact · Zed style ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::uniform(1));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // (width, height, arc_color, dim_color, spin, ticks_per_step, label)
    let configs: &[(usize, usize, Color, Color, Spin, u64, &str)] = &[
        (
            6,
            2,
            Color::Cyan,
            Color::DarkGray,
            Spin::Clockwise,
            1,
            "6×2  ↻  cyan",
        ),
        (
            8,
            2,
            Color::LightBlue,
            Color::DarkGray,
            Spin::CounterClockwise,
            1,
            "8×2  ↺  blue",
        ),
        (
            6,
            3,
            Color::Cyan,
            Color::DarkGray,
            Spin::Clockwise,
            2,
            "6×3  ↻  slow",
        ),
        (
            8,
            3,
            Color::LightCyan,
            Color::DarkGray,
            Spin::CounterClockwise,
            1,
            "8×3  ↺  cyan",
        ),
        (
            10,
            2,
            Color::White,
            Color::DarkGray,
            Spin::Clockwise,
            1,
            "10×2 ↻  white",
        ),
        (
            10,
            3,
            Color::Cyan,
            Color::Blue,
            Spin::CounterClockwise,
            2,
            "10×3 ↺  blue bg",
        ),
    ];

    let row_h = 1u16.max(inner.height / configs.len() as u16);
    let constraints: Vec<Constraint> = configs
        .iter()
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(w, h, arc, dim, spin, tps, label)) in configs.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let row = rows[i];
        let spinner_w = w as u16;
        let [spinner_area, label_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            RectangularSpinner::new(tick)
                .width(w)
                .height(h)
                .spin(spin)
                .arc_color(arc)
                .dim_color(dim)
                .ticks_per_step(tps),
            spinner_area,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(format!(" {label}"), Style::default().fg(arc))),
            label_area,
        );
    }
}

// ── Col 2 — Wide banner spinners (Claude style) ───────────────────────────────

fn render_wide_column(frame: &mut Frame, area: Rect, tick: u64) {
    let outer_block = Block::bordered()
        .title(" Wide Banner · Claude style ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Yellow))
        .padding(Padding::uniform(1));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Claude-ish warm/orange palette — wide, 2-row banners
    let configs: &[(usize, Color, Color, Spin, u64, &str)] = &[
        (
            18,
            Color::Yellow,
            Color::DarkGray,
            Spin::Clockwise,
            1,
            "↻ yellow",
        ),
        (
            18,
            Color::LightYellow,
            Color::DarkGray,
            Spin::CounterClockwise,
            1,
            "↺ lt-yellow",
        ),
        (
            18,
            Color::Rgb(255, 165, 0),
            Color::DarkGray,
            Spin::Clockwise,
            2,
            "↻ orange",
        ),
        (
            18,
            Color::Rgb(255, 200, 50),
            Color::Rgb(80, 40, 0),
            Spin::CounterClockwise,
            1,
            "↺ warm bg",
        ),
        (
            18,
            Color::White,
            Color::DarkGray,
            Spin::Clockwise,
            1,
            "↻ white",
        ),
        (
            18,
            Color::LightRed,
            Color::DarkGray,
            Spin::CounterClockwise,
            2,
            "↺ lt-red",
        ),
    ];

    let row_h = 2u16.max(inner.height / configs.len() as u16);
    let constraints: Vec<Constraint> = configs
        .iter()
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(w, arc, dim, spin, tps, label)) in configs.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let row = rows[i];

        // Clamp width to available space
        let avail_w = row.width.saturating_sub(12);
        let actual_w = w.min(avail_w as usize).max(3);
        let spinner_w = actual_w as u16;

        let [spinner_area, label_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            RectangularSpinner::new(tick)
                .width(actual_w)
                .height(2)
                .spin(spin)
                .arc_color(arc)
                .dim_color(dim)
                .ticks_per_step(tps),
            spinner_area,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(format!(" {label}"), Style::default().fg(arc))),
            label_area,
        );
    }
}

// ── Col 3 — Tall spinners (both directions) ───────────────────────────────────

fn render_tall_column(frame: &mut Frame, area: Rect, tick: u64) {
    let outer_block = Block::bordered()
        .title(" Tall · Both Directions ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Magenta))
        .padding(Padding::uniform(1));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // Side-by-side CW and CCW pairs
    let configs: &[(usize, usize, Color, Spin, u64)] = &[
        (6, 4, Color::Magenta, Spin::Clockwise, 1),
        (6, 4, Color::LightMagenta, Spin::CounterClockwise, 1),
        (8, 5, Color::Green, Spin::Clockwise, 2),
        (8, 5, Color::LightGreen, Spin::CounterClockwise, 2),
    ];

    let n = configs.len();
    let row_h = inner.height / n as u16;
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h.max(1)))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(w, h, color, spin, tps)) in configs.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let row = rows[i];
        let dir_label = if matches!(spin, Spin::Clockwise) {
            "↻ CW "
        } else {
            "↺ CCW"
        };

        let spinner_w = w as u16;
        let [spinner_area, label_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            RectangularSpinner::new(tick)
                .width(w)
                .height(h)
                .spin(spin)
                .arc_color(color)
                .dim_color(Color::DarkGray)
                .ticks_per_step(tps),
            spinner_area,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" {dir_label} {w}×{h}"),
                Style::default().fg(color),
            )),
            label_area,
        );
    }
}

// ── Col 4 — Custom arc lengths and speeds ────────────────────────────────────

fn render_custom_column(frame: &mut Frame, area: Rect, tick: u64) {
    let outer_block = Block::bordered()
        .title(" Custom Arc & Speed ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightRed))
        .padding(Padding::uniform(1));

    let inner = outer_block.inner(area);
    frame.render_widget(outer_block, area);

    // (width, height, arc_len, color, spin, ticks_per_step, label)
    let configs: &[(usize, usize, usize, Color, Spin, u64, &str)] = &[
        (8, 3, 4, Color::Red, Spin::Clockwise, 1, "arc=4  fast"),
        (
            8,
            3,
            8,
            Color::LightRed,
            Spin::CounterClockwise,
            1,
            "arc=8  fast",
        ),
        (8, 3, 12, Color::Yellow, Spin::Clockwise, 2, "arc=12 med"),
        (
            8,
            3,
            20,
            Color::Green,
            Spin::CounterClockwise,
            3,
            "arc=20 slow",
        ),
        (8, 3, 30, Color::Magenta, Spin::Clockwise, 4, "arc=30 vslow"),
        (8, 3, 0, Color::Cyan, Spin::CounterClockwise, 1, "arc=auto"),
    ];

    let row_h = 1u16.max(inner.height / configs.len() as u16);
    let constraints: Vec<Constraint> = configs
        .iter()
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(w, h, arc, color, spin, tps, label)) in configs.iter().enumerate() {
        if i >= rows.len() {
            break;
        }
        let row = rows[i];
        let spinner_w = w as u16;

        let [spinner_area, label_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            RectangularSpinner::new(tick)
                .width(w)
                .height(h)
                .arc_len(arc)
                .spin(spin)
                .arc_color(color)
                .dim_color(Color::DarkGray)
                .ticks_per_step(tps),
            spinner_area,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!(" {label}"),
                Style::default().fg(color),
            )),
            label_area,
        );
    }
}
