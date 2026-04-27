//! # ZedSpinner Example
//!
//! Demonstrates the [`ZedSpinner`] widget — a single braille character (or
//! block of characters) cycling through 8 frames where one dot is missing
//! and the gap rotates clockwise: `⣾ ⣷ ⣯ ⣟ ⡿ ⢿ ⣽ ⣻`.
//!
//! Layout:
//! - **Top-left**   — Single-character spinners at various speeds and colours
//! - **Top-right**  — Horizontal wave: same spinner repeated with phase offset
//! - **Bottom-left** — Grid (width × height) with diagonal wave
//! - **Bottom-right** — Simulated status-bar showing real-world usage
//!
//! **Controls:**
//! - `q` / `Esc` — Quit
//!
//! Run with: `cargo run --example zed_spinner`

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
use tui_spinner::ZedSpinner;

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
    let block = Block::bordered()
        .title(" ZedSpinner  ·  ⣾⣷⣯⣟⡿⢿⣽⣻  ·  rotating braille dot ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(
        Paragraph::new("1×1 minimal  ·  horizontal wave  ·  grid  ·  status-bar demo")
            .alignment(Alignment::Center)
            .style(Style::default().fg(Color::DarkGray))
            .block(block),
        area,
    );
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray));

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
        Paragraph::new(line)
            .alignment(Alignment::Center)
            .block(block),
        area,
    );
}

fn render_body(frame: &mut Frame, area: Rect, tick: u64) {
    let [top, bottom] =
        Layout::vertical([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area);

    let [single_area, wave_area] =
        Layout::horizontal([Constraint::Percentage(40), Constraint::Percentage(60)]).areas(top);

    let [grid_area, status_area] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(bottom);

    render_single_section(frame, single_area, tick);
    render_wave_section(frame, wave_area, tick);
    render_grid_section(frame, grid_area, tick);
    render_status_section(frame, status_area, tick);
}

// ── Single-character spinners ─────────────────────────────────────────────────

/// (color, ticks_per_step, label)
const SINGLE_CONFIGS: &[(Color, u64, &str)] = &[
    (Color::Cyan, 1, "cyan       1 tick/frame"),
    (Color::White, 1, "white      1 tick/frame"),
    (Color::LightBlue, 2, "lt-blue    2 ticks/frame"),
    (Color::Yellow, 1, "yellow     1 tick/frame"),
    (Color::Rgb(255, 165, 0), 1, "orange  1 tick/frame"),
    (Color::LightGreen, 3, "lt-green   3 ticks/frame"),
    (Color::Magenta, 1, "magenta    1 tick/frame"),
    (Color::LightRed, 4, "lt-red     4 ticks/frame"),
];

fn render_single_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .title(" Single character  1×1 ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan))
        .padding(Padding::horizontal(1));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = SINGLE_CONFIGS.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(color, tps, label)) in SINGLE_CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];
        let [spin_area, label_area] =
            Layout::horizontal([Constraint::Length(1), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            ZedSpinner::new(tick).color(color).ticks_per_step(tps),
            spin_area,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("  {label}"),
                Style::default().fg(color),
            )),
            label_area,
        );
    }
}

// ── Horizontal wave ───────────────────────────────────────────────────────────

/// (color, phase_step, ticks_per_step, label)
const WAVE_CONFIGS: &[(Color, u8, u64, &str)] = &[
    (Color::Cyan, 1, 1, "phase=1  smooth wave"),
    (Color::LightCyan, 2, 1, "phase=2  faster wave"),
    (Color::White, 3, 1, "phase=3  3-step wave"),
    (Color::Yellow, 4, 1, "phase=4  anti-phase pairs"),
    (Color::Rgb(255, 165, 0), 0, 1, "phase=0  synchronised"),
    (Color::LightGreen, 1, 2, "phase=1  slow"),
    (Color::Magenta, 1, 1, "phase=1  magenta"),
    (Color::LightBlue, 2, 1, "phase=2  blue"),
];

fn render_wave_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .title(" Horizontal wave  width(N) + phase_step ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightBlue))
        .padding(Padding::horizontal(1));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = WAVE_CONFIGS.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(color, phase, tps, label)) in WAVE_CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];
        // Reserve ~24 chars for the label, rest is spinner.
        let label_w = (label.len() as u16 + 2).min(row.width.saturating_sub(4));
        let spinner_w = row.width.saturating_sub(label_w);

        let [spin_area, label_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Length(label_w)])
                .areas(row);

        frame.render_widget(
            ZedSpinner::new(tick)
                .width(spinner_w as usize)
                .color(color)
                .phase_step(phase)
                .ticks_per_step(tps),
            spin_area,
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

// ── Grid (width × height) ─────────────────────────────────────────────────────

/// (cols, rows, color, phase_step, ticks_per_step, label)
const GRID_CONFIGS: &[(usize, usize, Color, u8, u64, &str)] = &[
    (3, 3, Color::Cyan, 1, 1, "3×3  phase=1"),
    (5, 2, Color::Yellow, 1, 1, "5×2  phase=1"),
    (4, 3, Color::Magenta, 2, 1, "4×3  phase=2"),
    (6, 2, Color::LightGreen, 1, 2, "6×2  slow"),
    (4, 2, Color::White, 0, 1, "4×2  sync"),
];

fn render_grid_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .title(" Grid  width(W).height(H) ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Magenta))
        .padding(Padding::horizontal(1));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = GRID_CONFIGS.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(w, h, color, phase, tps, label)) in GRID_CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];
        let spinner_w = w as u16;

        let [spin_area, label_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            ZedSpinner::new(tick)
                .width(w)
                .height(h)
                .color(color)
                .phase_step(phase)
                .ticks_per_step(tps),
            spin_area,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("  {label}"),
                Style::default().fg(color),
            )),
            label_area,
        );
    }
}

// ── Status-bar simulation ─────────────────────────────────────────────────────
//
// Shows how ZedSpinner would actually appear in a real status bar:
// a tiny 1×1 spinner next to a label, in various "task" contexts.

const STATUS_ITEMS: &[(Color, &str)] = &[
    (Color::Cyan, "Indexing workspace…"),
    (Color::LightBlue, "Loading extensions…"),
    (Color::Yellow, "Running diagnostics…"),
    (Color::Rgb(255, 165, 0), "Fetching completions…"),
    (Color::LightGreen, "Formatting file…"),
    (Color::Magenta, "Running tests…"),
    (Color::White, "Connecting to LSP…"),
    (Color::LightCyan, "Analyzing symbols…"),
];

fn render_status_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .title(" Status-bar simulation ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Fake "status bar" rows — each one cycles through tasks over time.
    let n = STATUS_ITEMS.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    // Stagger each row so they don't all pulse together.
    for (i, &(color, label)) in STATUS_ITEMS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];
        // Each row gets a tick offset so they're staggered.
        let row_tick = tick.wrapping_add((i as u64) * 3);

        let [spin_area, label_area] =
            Layout::horizontal([Constraint::Length(1), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            ZedSpinner::new(row_tick).color(color).ticks_per_step(2),
            spin_area,
        );
        frame.render_widget(
            Paragraph::new(Span::styled(
                format!("  {label}"),
                Style::default().fg(color),
            )),
            label_area,
        );
    }
}
