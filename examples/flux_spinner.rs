//! # FluxSpinner Example
//!
//! Demonstrates the [`FluxSpinner`] widget — a compact braille rotation
//! spinner where one dot is missing and the gap rotates clockwise or
//! counter-clockwise: `⣾ ⣷ ⣯ ⣟ ⡿ ⢿ ⣽ ⣻`
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
use tui_spinner::{FluxSpinner, Spin};

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

// ── Layout ────────────────────────────────────────────────────────────────────

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
        Paragraph::new("⣾ ⣷ ⣯ ⣟ ⡿ ⢿ ⣽ ⣻")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray))
            .block(
                Block::bordered()
                    .title(" FluxSpinner ")
                    .title_alignment(Alignment::Center)
                    .border_type(BorderType::Rounded)
                    .border_style(Style::default().fg(Color::Cyan)),
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

// ── Spinner configs ───────────────────────────────────────────────────────────

/// (color, spin, ticks_per_step, label)
const CONFIGS: &[(Color, Spin, u64, &str)] = &[
    (Color::Cyan, Spin::Clockwise, 1, "cyan    1t"),
    (Color::White, Spin::Clockwise, 1, "white   1t"),
    (Color::LightBlue, Spin::CounterClockwise, 2, "lt-blue 2t"),
    (Color::Yellow, Spin::Clockwise, 1, "yellow  1t"),
    (
        Color::Rgb(255, 165, 0),
        Spin::CounterClockwise,
        1,
        "orange  1t",
    ),
    (Color::LightGreen, Spin::Clockwise, 3, "lt-grn  3t"),
    (Color::Magenta, Spin::CounterClockwise, 1, "magenta 1t"),
    (Color::LightRed, Spin::Clockwise, 4, "lt-red  4t"),
];

fn render_body(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .title(" 1×1  ·  CW ↻ / CCW ↺ ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::horizontal(1));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = CONFIGS.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(color, spin, tps, label)) in CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];

        // spinner | label | direction symbol
        let dir_symbol = if matches!(spin, Spin::Clockwise) {
            "↻"
        } else {
            "↺"
        };
        let full_label = format!("  {label}  {dir_symbol}");

        let [spin_area, label_area] =
            Layout::horizontal([Constraint::Length(1), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            FluxSpinner::new(tick)
                .color(color)
                .spin(spin)
                .ticks_per_step(tps),
            spin_area,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(full_label, Style::default().fg(color))),
            label_area,
        );
    }
}
