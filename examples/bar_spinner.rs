//! # BarSpinner Example
//!
//! Demonstrates the [`BarSpinner`] widget — a Zed / Claude-style
//! braille loading bar where a glowing arc bounces left and right.
//!
//! Every row spans the **full column width** so the animation looks exactly
//! like real Zed or Claude loading indicators (not tiny box outlines).
//!
//! Layout (top → bottom):
//! - **Zed-style**  — 1-row bars, cyan / blue palette
//! - **Claude-style** — 2-row bars, warm orange / yellow palette
//! - **Thick**      — 3-row bars
//! - **Pairs**      — CW + CCW side-by-side at the same tick so you can see
//!                    the ping-pong offset between the two directions
//! - **Arc widths** — narrow → wide arcs on the same bar width
//!
//! **Controls:**
//! - `q` / `Esc` — Quit
//!
//! Run with: `cargo run --example bar_spinner`

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
use tui_spinner::{BarSpinner, BarTrack, Spin};

// ── Style macros ──────────────────────────────────────────────────────────────

/// Build a [`Style`] quickly.
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

/// Build a rounded bordered [`Block`] with a centred title and padding.
macro_rules! section_block {
    ($title:expr, $color:expr) => {
        Block::bordered()
            .title(concat!(" ", $title, " "))
            .title_alignment(Alignment::Center)
            .border_type(BorderType::Rounded)
            .border_style(sty!($color))
            .padding(Padding::uniform(1))
    };
}

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
        .title(" BarSpinner — Zed / Claude-style bouncing bar ")
        .title_alignment(Alignment::Center)
        .border_type(BorderType::Rounded)
        .border_style(sty!(Color::Cyan));

    let subtitle = Paragraph::new("full-width bars · gradient arc · ping-pong bounce")
        .alignment(Alignment::Center)
        .style(sty!(dim))
        .block(block);

    frame.render_widget(subtitle, area);
}

fn render_footer(frame: &mut Frame, area: Rect) {
    let block = Block::bordered()
        .border_type(BorderType::Rounded)
        .border_style(sty!(dim));

    let line = Line::from(vec![
        sp!("q"; Color::Cyan, b),
        Span::raw(" / "),
        sp!("Esc"; Color::Cyan, b),
        Span::raw("  Quit"),
    ]);

    frame.render_widget(
        Paragraph::new(line)
            .alignment(Alignment::Center)
            .block(block),
        area,
    );
}

// ── Body — two-column layout ──────────────────────────────────────────────────
//
//  Left  (60 %): stacked full-width bars with labels
//  Right (40 %): arc-width comparison + pair demo

fn render_body(frame: &mut Frame, area: Rect, tick: u64) {
    let [left, right] =
        Layout::horizontal([Constraint::Percentage(60), Constraint::Percentage(40)]).areas(area);

    render_left(frame, left, tick);
    render_right(frame, right, tick);
}

// ── Left panel ────────────────────────────────────────────────────────────────

fn render_left(frame: &mut Frame, area: Rect, tick: u64) {
    // Three sections stacked: Zed / Claude / Thick
    let [zed_area, claude_area, thick_area] = Layout::vertical([
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
        Constraint::Ratio(1, 3),
    ])
    .areas(area);

    render_zed_section(frame, zed_area, tick);
    render_claude_section(frame, claude_area, tick);
    render_thick_section(frame, thick_area, tick);
}

// ── Zed-style section (1-row bars) ───────────────────────────────────────────

/// (arc_color, dim_color, spin, ticks_per_step, label)
const ZED_CONFIGS: &[(Color, Color, Spin, u64, &str)] = &[
    (Color::Cyan, Color::DarkGray, Spin::Clockwise, 1, "cyan   ↻"),
    (
        Color::LightBlue,
        Color::DarkGray,
        Spin::CounterClockwise,
        1,
        "blue   ↺",
    ),
    (
        Color::White,
        Color::DarkGray,
        Spin::Clockwise,
        2,
        "white  ↻  slow",
    ),
    (
        Color::LightCyan,
        Color::Rgb(0, 30, 40),
        Spin::CounterClockwise,
        1,
        "cyan   ↺  dark bg",
    ),
    (
        Color::Cyan,
        Color::Black,
        Spin::Clockwise,
        1,
        "cyan   ↻  no track",
    ),
];

fn render_zed_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("Zed-style  ·  1 row  ·  height(1)", Color::Cyan);

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = ZED_CONFIGS.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(arc, dim, spin, tps, label)) in ZED_CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        render_bar_row(frame, rows[i], tick, 1, arc, dim, spin, tps, label);
    }
}

// ── Claude-style section (2-row bars) ────────────────────────────────────────

const CLAUDE_CONFIGS: &[(Color, Color, Spin, u64, &str)] = &[
    (
        Color::Yellow,
        Color::DarkGray,
        Spin::Clockwise,
        1,
        "yellow   ↻",
    ),
    (
        Color::LightYellow,
        Color::DarkGray,
        Spin::CounterClockwise,
        1,
        "lt-yel   ↺",
    ),
    (
        Color::Rgb(255, 165, 0),
        Color::DarkGray,
        Spin::Clockwise,
        2,
        "orange   ↻  slow",
    ),
    (
        Color::Rgb(255, 200, 50),
        Color::Rgb(60, 30, 0),
        Spin::CounterClockwise,
        1,
        "amber    ↺  warm bg",
    ),
];

fn render_claude_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("Claude-style  ·  2 rows  ·  height(2)", Color::Yellow);

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = CLAUDE_CONFIGS.len();
    let row_h = (inner.height / n as u16).max(2);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(arc, dim, spin, tps, label)) in CLAUDE_CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        render_bar_row(frame, rows[i], tick, 2, arc, dim, spin, tps, label);
    }
}

// ── Thick section (3-row bars) ───────────────────────────────────────────────

const THICK_CONFIGS: &[(Color, Color, Spin, u64, &str)] = &[
    (
        Color::Green,
        Color::DarkGray,
        Spin::Clockwise,
        1,
        "green    ↻",
    ),
    (
        Color::LightGreen,
        Color::DarkGray,
        Spin::CounterClockwise,
        1,
        "lt-green ↺",
    ),
    (
        Color::Magenta,
        Color::DarkGray,
        Spin::Clockwise,
        2,
        "magenta  ↻  slow",
    ),
];

fn render_thick_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("Thick  ·  3 rows  ·  height(3)", Color::Green);

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = THICK_CONFIGS.len();
    let row_h = (inner.height / n as u16).max(3);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(arc, dim, spin, tps, label)) in THICK_CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        render_bar_row(frame, rows[i], tick, 3, arc, dim, spin, tps, label);
    }
}

// ── Right panel ───────────────────────────────────────────────────────────────

fn render_right(frame: &mut Frame, area: Rect, tick: u64) {
    let [pairs_area, arc_area, track_area, fade_area] = Layout::vertical([
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
    ])
    .areas(area);

    render_pairs_section(frame, pairs_area, tick);
    render_arc_section(frame, arc_area, tick);
    render_track_section(frame, track_area, tick);
    render_fade_section(frame, fade_area, tick);
}

// ── CW / CCW pairs ────────────────────────────────────────────────────────────
//
// Two bars at the same tick but opposite start directions — you can see them
// moving away from each other and bouncing back in sync.

const PAIR_CONFIGS: &[(Color, u64, &str)] = &[
    (Color::Cyan, 1, "cyan"),
    (Color::Yellow, 1, "yellow"),
    (Color::Magenta, 2, "magenta"),
];

fn render_pairs_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("CW ↻ vs CCW ↺  ·  same tick", Color::LightCyan);

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    // Each pair takes two rows (CW on top, CCW below) + 1 gap.
    let n = PAIR_CONFIGS.len();
    let pair_h = (inner.height / n as u16).max(2);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(pair_h))
        .chain([Constraint::Min(0)])
        .collect();
    let slots = Layout::vertical(constraints).split(inner);

    for (i, &(color, tps, label)) in PAIR_CONFIGS.iter().enumerate() {
        if i >= slots.len().saturating_sub(1) {
            break;
        }
        let slot = slots[i];

        // Split each slot: top row = CW, bottom row = CCW.
        let [cw_row, ccw_row] =
            Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(slot);

        render_bar_row(
            frame,
            cw_row,
            tick,
            1,
            color,
            Color::DarkGray,
            Spin::Clockwise,
            tps,
            &format!("{label} ↻"),
        );
        render_bar_row(
            frame,
            ccw_row,
            tick,
            1,
            color,
            Color::DarkGray,
            Spin::CounterClockwise,
            tps,
            &format!("{label} ↺"),
        );
    }
}

// ── Track styles ─────────────────────────────────────────────────────────────

/// (track, label)
const TRACK_CONFIGS: &[(BarTrack, &str)] = &[
    (BarTrack::Rail, "Rail (default)  ⣀"),
    (BarTrack::Full, "Full            ⣿"),
    (BarTrack::Empty, "Empty           ⠀"),
    (BarTrack::Custom(0x09), "Custom  0x09    ⠉"),
];

fn render_track_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("Track style  .track(BarTrack::…)", Color::LightYellow);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = TRACK_CONFIGS.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(track, label)) in TRACK_CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];
        let label_w = label.len() as u16 + 2;
        let [spinner_area, label_area] =
            Layout::horizontal([Constraint::Min(4), Constraint::Length(label_w)]).areas(row);

        frame.render_widget(
            BarSpinner::new(tick)
                .height(1)
                .track(track)
                .arc_color(Color::Yellow)
                .dim_color(Color::DarkGray)
                .ticks_per_step(1),
            spinner_area,
        );
        frame.render_widget(Paragraph::new(sp!(format!(" {label}"); dim)), label_area);
    }
}

// ── Fade width comparison ─────────────────────────────────────────────────────

/// (fade_width, label)
const FADE_CONFIGS: &[(usize, &str)] = &[
    (0, "fade=0  sharp ⣿"),
    (1, "fade=1  subtle"),
    (2, "fade=2"),
    (3, "fade=3  default ⠉⠛⠿⣿"),
];

fn render_fade_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("Fade width  .fade_width(n)", Color::LightMagenta);
    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = FADE_CONFIGS.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(fw, label)) in FADE_CONFIGS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];
        let label_w = label.len() as u16 + 2;
        let [spinner_area, label_area] =
            Layout::horizontal([Constraint::Min(4), Constraint::Length(label_w)]).areas(row);

        frame.render_widget(
            BarSpinner::new(tick)
                .height(1)
                .fade_width(fw)
                .arc_color(Color::Magenta)
                .dim_color(Color::DarkGray)
                .ticks_per_step(1),
            spinner_area,
        );
        frame.render_widget(Paragraph::new(sp!(format!(" {label}"); dim)), label_area);
    }
}

// ── Arc-width comparison ──────────────────────────────────────────────────────

/// (arc_width_chars, label)
const ARC_WIDTHS: &[(usize, &str)] = &[
    (3, "arc=3  narrow"),
    (6, "arc=6"),
    (10, "arc=10"),
    (0, "arc=auto"),
];

fn render_arc_section(frame: &mut Frame, area: Rect, tick: u64) {
    let outer = section_block!("Arc width comparison", Color::LightRed);

    let inner = outer.inner(area);
    frame.render_widget(outer, area);

    let n = ARC_WIDTHS.len();
    let row_h = (inner.height / n as u16).max(1);
    let constraints: Vec<Constraint> = (0..n)
        .map(|_| Constraint::Length(row_h))
        .chain([Constraint::Min(0)])
        .collect();
    let rows = Layout::vertical(constraints).split(inner);

    for (i, &(arc_w, label)) in ARC_WIDTHS.iter().enumerate() {
        if i >= rows.len().saturating_sub(1) {
            break;
        }
        let row = rows[i];
        // Label on the right; spinner fills the remaining width.
        let label_w = label.len() as u16 + 2;
        let [spinner_area, label_area] =
            Layout::horizontal([Constraint::Min(4), Constraint::Length(label_w)]).areas(row);

        frame.render_widget(
            BarSpinner::new(tick)
                .arc_width(arc_w)
                .height(1)
                .spin(Spin::Clockwise)
                .arc_color(Color::LightRed)
                .dim_color(Color::DarkGray)
                .ticks_per_step(1),
            spinner_area,
        );
        frame.render_widget(Paragraph::new(sp!(format!(" {label}"); dim)), label_area);
    }
}

// ── Shared helper: render one full-width bar + right-aligned label ────────────

fn render_bar_row(
    frame: &mut Frame,
    row: Rect,
    tick: u64,
    height: usize,
    arc_color: Color,
    dim_color: Color,
    spin: Spin,
    ticks_per_step: u64,
    label: &str,
) {
    // Reserve just enough space on the right for the label.
    let label_w = (label.len() as u16 + 2).min(row.width.saturating_sub(4));
    let spinner_w = row.width.saturating_sub(label_w);

    let [spinner_area, label_area] =
        Layout::horizontal([Constraint::Length(spinner_w), Constraint::Length(label_w)]).areas(row);

    frame.render_widget(
        BarSpinner::new(tick)
            .height(height)
            // width = 0 → fill spinner_area automatically
            .spin(spin)
            .arc_color(arc_color)
            .dim_color(dim_color)
            .ticks_per_step(ticks_per_step),
        spinner_area,
    );

    // Vertically centre the label within the row.
    let label_row = if row.height > 1 {
        let [_, mid, _] = Layout::vertical([
            Constraint::Length(row.height / 2),
            Constraint::Length(1),
            Constraint::Min(0),
        ])
        .areas(label_area);
        mid
    } else {
        label_area
    };

    frame.render_widget(
        Paragraph::new(sp!(format!(" {label}"); arc_color)),
        label_row,
    );
}
