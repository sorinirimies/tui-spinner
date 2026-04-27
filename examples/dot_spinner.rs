//! # DotSpinner Example
//!
//! Demonstrates the [`DotSpinner`] widget — a compact braille rotation spinner.
//! Each character cell cycles through 8 frames where one dot is missing and
//! the gap rotates: `⣾ ⣷ ⣯ ⣟ ⡿ ⢿ ⣽ ⣻` (clockwise) or reversed (CCW).
//!
//! Layout:
//! - **Top-left**    — 1×1 single-char: various colours, CW and CCW
//! - **Top-right**   — Direction: CW ↻ vs CCW ↺ side-by-side at the same tick
//! - **Bottom-left** — Horizontal wave: width(N) + phase_step
//! - **Bottom-right** — Grid (W×H) and status-bar simulation
//!
//! **Controls:**
//! - `q` / `Esc` — Quit
//!
//! Run with: `cargo run --example dot_spinner`

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
use tui_spinner::{DotSpinner, Spin};

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
        .title(" DotSpinner  ·  ⣾⣷⣯⣟⡿⢿⣽⣻  ·  rotating braille dot ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::Cyan));

    frame.render_widget(
        Paragraph::new("1×1 single  ·  CW ↻ vs CCW ↺  ·  wave  ·  grid + status-bar")
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

    // Top: single-char demo (left) + direction comparison (right)
    let [single_area, dir_area] =
        Layout::horizontal([Constraint::Percentage(38), Constraint::Percentage(62)]).areas(top);

    // Bottom: wave (left) + grid & status-bar (right split)
    let [wave_area, right_bottom] =
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(bottom);

    let [grid_area, status_area] =
        Layout::vertical([Constraint::Percentage(55), Constraint::Percentage(45)])
            .areas(right_bottom);

    render_single_section(frame, single_area, tick);
    render_direction_section(frame, dir_area, tick);
    render_wave_section(frame, wave_area, tick);
    render_grid_section(frame, grid_area, tick);
    render_status_section(frame, status_area, tick);
}

// ── 1×1 single-character spinners ────────────────────────────────────────────

/// (color, spin, ticks_per_step, label)
const SINGLE_CONFIGS: &[(Color, Spin, u64, &str)] = &[
    (Color::Cyan, Spin::Clockwise, 1, "cyan    ↻  1t"),
    (Color::White, Spin::Clockwise, 1, "white   ↻  1t"),
    (Color::LightBlue, Spin::CounterClockwise, 2, "lt-blue ↺  2t"),
    (Color::Yellow, Spin::Clockwise, 1, "yellow  ↻  1t"),
    (
        Color::Rgb(255, 165, 0),
        Spin::CounterClockwise,
        1,
        "orange  ↺  1t",
    ),
    (Color::LightGreen, Spin::Clockwise, 3, "lt-grn  ↻  3t"),
    (Color::Magenta, Spin::CounterClockwise, 1, "magenta ↺  1t"),
    (Color::LightRed, Spin::Clockwise, 4, "lt-red  ↻  4t"),
];

fn render_single_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .title(" 1×1  ·  CW ↻ / CCW ↺ ")
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

    for (i, &(color, spin, tps, label)) in SINGLE_CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];
        let [spin_area, label_area] =
            Layout::horizontal([Constraint::Length(1), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            DotSpinner::new(tick)
                .color(color)
                .spin(spin)
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

// ── Direction comparison ──────────────────────────────────────────────────────
// CW vs CCW at the same tick so the mirrored rotation and reversed wave
// direction are obvious.

const DIR_CONFIGS: &[(Color, u8, u64)] = &[
    (Color::Cyan, 1, 1),
    (Color::Yellow, 1, 1),
    (Color::LightGreen, 1, 1),
    (Color::Magenta, 2, 1),
    (Color::White, 0, 1),
];

fn render_direction_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .title(" CW vs CCW - same tick ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::LightCyan))
        .padding(Padding::horizontal(1));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = DIR_CONFIGS.len();
    let row_h = (inner.height.saturating_sub(1) / n as u16).max(1);
    let header_and_rows: Vec<Constraint> = std::iter::once(Constraint::Length(1))
        .chain((0..n).map(|_| Constraint::Length(row_h)))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(header_and_rows).split(inner);

    let half = inner.width / 2;
    let [lbl_cw, lbl_ccw] =
        Layout::horizontal([Constraint::Length(half), Constraint::Min(0)]).areas(rows[0]);
    frame.render_widget(
        Paragraph::new(Span::styled(
            " Clockwise",
            Style::default().fg(Color::DarkGray),
        )),
        lbl_cw,
    );
    frame.render_widget(
        Paragraph::new(Span::styled(
            " Counter-CW",
            Style::default().fg(Color::DarkGray),
        )),
        lbl_ccw,
    );

    for (i, &(color, phase, tps)) in DIR_CONFIGS.iter().enumerate() {
        if i + 1 >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i + 1];
        let spinner_w = (half.saturating_sub(1)) as usize;

        let [cw_area, ccw_area] =
            Layout::horizontal([Constraint::Length(half), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            DotSpinner::new(tick)
                .width(spinner_w)
                .spin(Spin::Clockwise)
                .color(color)
                .phase_step(phase)
                .ticks_per_step(tps),
            cw_area,
        );
        frame.render_widget(
            DotSpinner::new(tick)
                .width(spinner_w)
                .spin(Spin::CounterClockwise)
                .color(color)
                .phase_step(phase)
                .ticks_per_step(tps),
            ccw_area,
        );
    }
}

// ── Horizontal wave ───────────────────────────────────────────────────────────

const WAVE_CONFIGS: &[(Color, Spin, u8, u64, &str)] = &[
    (Color::Cyan, Spin::Clockwise, 1, 1, "phase=1 CW"),
    (
        Color::LightCyan,
        Spin::CounterClockwise,
        1,
        1,
        "phase=1 CCW",
    ),
    (Color::Yellow, Spin::Clockwise, 2, 1, "phase=2 CW"),
    (
        Color::Rgb(255, 165, 0),
        Spin::CounterClockwise,
        2,
        1,
        "phase=2 CCW",
    ),
    (Color::LightGreen, Spin::Clockwise, 4, 1, "phase=4 CW"),
    (Color::Magenta, Spin::Clockwise, 0, 1, "phase=0 sync"),
];

fn render_wave_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .title(" Wave  width(N) + phase_step ")
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

    for (i, &(color, spin, phase, tps, label)) in WAVE_CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];
        let label_w = (label.len() as u16 + 2).min(row.width.saturating_sub(4));
        let spinner_w = row.width.saturating_sub(label_w);

        let [spin_area, label_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Length(label_w)])
                .areas(row);

        frame.render_widget(
            DotSpinner::new(tick)
                .width(spinner_w as usize)
                .color(color)
                .spin(spin)
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

// ── Grid ──────────────────────────────────────────────────────────────────────

const GRID_CONFIGS: &[(usize, usize, Color, Spin, u8, u64, &str)] = &[
    (3, 2, Color::Cyan, Spin::Clockwise, 1, 1, "3x2 CW p=1"),
    (
        4,
        2,
        Color::Yellow,
        Spin::CounterClockwise,
        1,
        1,
        "4x2 CCW p=1",
    ),
    (3, 3, Color::Magenta, Spin::Clockwise, 2, 1, "3x3 CW p=2"),
    (
        5,
        2,
        Color::LightGreen,
        Spin::CounterClockwise,
        2,
        1,
        "5x2 CCW p=2",
    ),
    (4, 2, Color::White, Spin::Clockwise, 0, 1, "4x2 sync"),
];

fn render_grid_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .title(" Grid  WxH + direction ")
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

    for (i, &(w, h, color, spin, phase, tps, label)) in GRID_CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];
        let spinner_w = w as u16;

        let [spin_area, label_area] =
            Layout::horizontal([Constraint::Length(spinner_w), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            DotSpinner::new(tick)
                .width(w)
                .height(h)
                .color(color)
                .spin(spin)
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

const STATUS_ITEMS: &[(Color, Spin, &str)] = &[
    (Color::Cyan, Spin::Clockwise, "Indexing workspace"),
    (
        Color::LightBlue,
        Spin::CounterClockwise,
        "Loading extensions",
    ),
    (Color::Yellow, Spin::Clockwise, "Running diagnostics"),
    (Color::LightGreen, Spin::Clockwise, "Formatting file"),
    (Color::Magenta, Spin::CounterClockwise, "Running tests"),
    (Color::White, Spin::Clockwise, "Connecting to LSP"),
];

fn render_status_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = Block::bordered()
        .title(" Status-bar demo ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(Style::default().fg(Color::DarkGray))
        .padding(Padding::horizontal(1));

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = STATUS_ITEMS.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(color, spin, label)) in STATUS_ITEMS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];
        let row_tick = tick.wrapping_add((i as u64) * 3);

        let [spin_area, label_area] =
            Layout::horizontal([Constraint::Length(1), Constraint::Min(0)]).areas(row);

        frame.render_widget(
            DotSpinner::new(row_tick)
                .color(color)
                .spin(spin)
                .ticks_per_step(2),
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
