//! # FluxSpinner Example
//!
//! Demonstrates the [`FluxSpinner`] widget — a compact braille rotation
//! spinner where one dot is missing and the gap rotates clockwise or
//! counter-clockwise.
//!
//! Layout:
//! - **Left column**  — Clockwise ↻ : 8 colours at varying speeds
//! - **Right column** — Counter-CW ↺ : same 8 colours, mirrored rotation
//! - **Bottom strip** — All [`FluxFrames`] presets side by side
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

// ── Layout ────────────────────────────────────────────────────────────────────

fn render(frame: &mut Frame, app: &App) {
    let [header, body, presets, footer] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Min(0),
        Constraint::Length(5),
        Constraint::Length(3),
    ])
    .areas(frame.area());

    render_header(frame, header);
    render_body(frame, body, app.tick);
    render_presets(frame, presets, app.tick);
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

// ── Main body: two columns ────────────────────────────────────────────────────
//
// Left  = Clockwise ↻     Right = Counter-Clockwise ↺
// Same colours and speeds on both sides so the mirrored rotation is obvious.

/// (color, ticks_per_step, short label)
const CONFIGS: &[(Color, u64, &str)] = &[
    (Color::Cyan, 4, "cyan"),
    (Color::White, 4, "white"),
    (Color::LightBlue, 5, "lt-blue"),
    (Color::Yellow, 4, "yellow"),
    (Color::Rgb(255, 165, 0), 4, "orange"),
    (Color::LightGreen, 6, "lt-green"),
    (Color::Magenta, 4, "magenta"),
    (Color::LightRed, 8, "lt-red"),
];

fn render_body(frame: &mut Frame, area: Rect, tick: u64) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

    render_direction_column(
        frame,
        left,
        tick,
        Spin::Clockwise,
        "↻  Clockwise",
        Color::Cyan,
    );
    render_direction_column(
        frame,
        right,
        tick,
        Spin::CounterClockwise,
        "↺  Counter-Clockwise",
        Color::LightBlue,
    );
}

fn render_direction_column(
    frame: &mut Frame,
    area: Rect,
    tick: u64,
    spin: Spin,
    title: &str,
    border_color: Color,
) {
    let outer = Block::bordered()
        .title(format!(" {title} "))
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(border_color))
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

    for (i, &(color, tps, label)) in CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];

        // Spinner | label | speed hint
        let hint = format!("  {}  {}t", label, tps);

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
            Paragraph::new(Span::styled(hint, Style::default().fg(color))),
            label_area,
        );
    }
}

// ── Presets strip ─────────────────────────────────────────────────────────────
//
// Shows every FluxFrames preset side by side so the different symbol sets
// are easy to compare at a glance.

/// (preset, name, color)
const PRESETS: &[(&'static [char], &str, Color)] = &[
    (FluxFrames::BRAILLE, "BRAILLE", Color::Cyan),
    (FluxFrames::ORBIT, "ORBIT", Color::LightBlue),
    (FluxFrames::CLASSIC, "CLASSIC", Color::Yellow),
    (FluxFrames::LINE, "LINE", Color::Rgb(255, 165, 0)),
    (FluxFrames::BLOCK, "BLOCK", Color::LightGreen),
    (FluxFrames::ARC, "ARC", Color::Magenta),
];

fn render_presets(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .title(" Frame presets — .frames(FluxFrames::…) ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Equal-width columns, one per preset.
    let n = PRESETS.len();
    let constraints: Vec<Constraint> = (0..n).map(|_| Constraint::Ratio(1, n as u32)).collect();
    let cols = Layout::horizontal(constraints).split(inner);

    // Two rows inside each column: spinner on top, name below.
    for (i, &(preset, name, color)) in PRESETS.iter().enumerate() {
        if i >= cols.len() {
            break;
        }
        let col = cols[i];

        let [spin_row, name_row] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(col);

        frame.render_widget(
            FluxSpinner::new(tick)
                .frames(preset)
                .color(color)
                .ticks_per_step(4)
                .alignment(Alignment::Center),
            spin_row,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(name, Style::default().fg(Color::DarkGray)))
                .alignment(Alignment::Center),
            name_row,
        );
    }
}
